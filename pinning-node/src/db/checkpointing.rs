use crate::types::keytable::{FaultTolerantKeyTable, TableRow};
use anyhow::Result;
use api::{common_types::BlockNumber, pinning_committee_types::NodeId};
use codec::{Decode, Encode};
use sled::{Batch as DbBatch, Db};

#[derive(Encode, Decode, Clone)]
pub struct Checkpoint {
    /// The block checkpoint. Holds the block informations until which the node has processed events.
    block_num: BlockNumber,
    /// The keytable managed by the pinning node, up to date with the block number.
    keytable: FaultTolerantKeyTable,
}

impl Checkpoint {
    pub fn new(block_num: BlockNumber, keytable: FaultTolerantKeyTable) -> Self {
        Checkpoint {
            block_num,
            keytable,
        }
    }

    pub fn height(&self) -> BlockNumber {
        self.block_num
    }

    pub fn keytable(self) -> FaultTolerantKeyTable {
        self.keytable
    }
}

pub struct DbCheckpoint {
    db: Db,
    rep_factor: u32,
    keytable_out_file: Option<String>,
}

impl DbCheckpoint {
    pub fn from_config(
        rep_factor: u32,
        node_id: NodeId,
        virtual_node_idx: u32,
        keytable_out_file: Option<String>,
    ) -> Self {
        // Open database
        let db = Self::open_db_from_node(virtual_node_idx, node_id);
        Self {
            db,
            rep_factor,
            keytable_out_file,
        }
    }

    fn open_db_from_node(idx: u32, node_id: NodeId) -> Db {
        let home = std::env::var("HOME").unwrap();
        let node_id = hex::encode(&node_id.encode()[..=8]);
        let db_name = format!("{}/virtual_{}/db_{}", home, idx, node_id);
        sled::open(db_name).unwrap()
    }

    /// Retrieves some checkpoint value from the database.
    fn read_checkpoint_value<D: Decode>(&self, key: &str) -> Result<Option<D>> {
        let checkpoint = self
            .db
            .get(key.as_bytes())
            .map_err(|_| anyhow::anyhow!("Failed to read checkpoint from db"))?;

        if let Some(checkpoint) = checkpoint {
            let checkpoint = D::decode(&mut checkpoint.as_ref())
                .map_err(|_| anyhow::anyhow!("Failed to decode checkpoint"))?;

            Ok(Some(checkpoint))
        } else {
            Ok(None)
        }
    }

    /// Retrieves the checkpoint.
    pub fn get_checkpoint(&self) -> Result<Checkpoint> {
        // Build the keytable
        let mut keytable =
            FaultTolerantKeyTable::new(self.rep_factor, self.keytable_out_file.clone());
        for idx in 0..self.rep_factor {
            let key = format!("partition_{}", idx);
            let row = self.read_checkpoint_value::<TableRow>(&key)?;

            if let Some(row) = row {
                keytable.mutable_table().add_row(row);
            }
        }

        let block_num = self.read_blocknumber()?.unwrap_or_default();

        Ok(Checkpoint::new(block_num, keytable))
    }

    /// Commits to storage the block number that the node has processed in terms of events and the affected rows in the keytable.
    pub fn commit_checkpoint(&self, block_num: BlockNumber, rows: Vec<&TableRow>) -> Result<()> {
        let mut batch = DbBatch::default();
        batch.insert(BLOCK_NUM_KEY, block_num.encode());
        for (idx, row) in rows.iter().enumerate() {
            let key = format!("partition_{}", idx);
            batch.insert(key.as_bytes(), row.encode());
        }

        // Commit the batch
        self.db.apply_batch(batch)?;

        Ok(())
    }

    pub fn read_blocknumber(&self) -> Result<Option<BlockNumber>> {
        let block_num = self.read_checkpoint_value::<BlockNumber>(BLOCK_NUM_KEY)?;

        Ok(block_num)
    }
}

pub const BLOCK_NUM_KEY: &str = "block_num";
