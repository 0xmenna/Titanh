use crate::types::keytable::{FaultTolerantKeyTable, TableRow};
use anyhow::Result;
use api::{common_types::BlockNumber, pinning_committee_types::NodeId};
use codec::{Decode, Encode};
use sled::{Batch as DbBatch, Db};

#[derive(Encode, Decode, Clone)]
pub struct Checkpoint {
    /// The block number checkpoint. Holds the block number until which the node has processed events.
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

    pub fn at(&self) -> BlockNumber {
        self.block_num
    }

    pub fn keytable(self) -> FaultTolerantKeyTable {
        self.keytable
    }
}

pub struct DbCheckpoint {
    db: Db,
    rep_factor: u32,
}

impl DbCheckpoint {
    pub fn from_config(rep_factor: u32, node_id: NodeId) -> Self {
        // Open database
        let db = Self::open_db_from_node(node_id);
        Self { db, rep_factor }
    }

    fn open_db_from_node(node_id: NodeId) -> Db {
        let db_name = format!("db_{}", node_id);
        sled::open(db_name).unwrap()
    }

    // This is unbounded to the struct instance because we still don't know the `rep_factor`, since it's a value fetched remotely, based on the block number, this is why we read this first.
    pub fn get_blocknumber_from_db_node(node_id: NodeId) -> Option<BlockNumber> {
        let db = Self::open_db_from_node(node_id);
        let block_num = Self::read_blocknumber(&db).expect("Failed to retrieve block number");

        block_num
    }

    /// Retrieves some checkpoint value from the database.
    fn read_checkpoint_value<D: Decode>(db: &Db, key: &str) -> Result<Option<D>> {
        let checkpoint = db
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
        let mut keytable = FaultTolerantKeyTable::new(self.rep_factor);
        for idx in 0..self.rep_factor {
            let key = format!("partition_{}", idx);
            let row = Self::read_checkpoint_value::<TableRow>(&self.db, &key)?;

            if let Some(row) = row {
                keytable.mutable_table().add_row(row);
            }
        }

        let block_num = Self::read_blocknumber(&self.db)?.unwrap_or_default();

        Ok(Checkpoint::new(block_num, keytable))
    }

    /// Commits to storage the block number that the node has processed in terms of events and the affected rows in the keytable.
    pub fn commit_checkpoint(&self, block_num: BlockNumber, rows: Vec<&TableRow>) -> Result<()> {
        let mut batch = DbBatch::default();
        batch.insert("block_num", block_num.encode());
        for (idx, row) in rows.iter().enumerate() {
            let key = format!("partition_{}", idx);
            batch.insert(key.as_bytes(), row.encode());
        }

        // Commit the batch
        self.db.apply_batch(batch)?;

        Ok(())
    }

    fn read_blocknumber(db: &Db) -> Result<Option<BlockNumber>> {
        let block_num = Self::read_checkpoint_value::<BlockNumber>(db, "block_num")?;

        Ok(block_num)
    }
}
