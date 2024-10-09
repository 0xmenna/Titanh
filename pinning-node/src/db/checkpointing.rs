use crate::{
    types::{
        cid::Cid,
        keytable::{FaultTolerantKeyTable, TableRow},
    },
    utils::config::Config,
};
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
    /// The pin counts for each CID.
    pin_counts: Vec<(Cid, u32)>,
}

impl Checkpoint {
    pub fn new(
        block_num: BlockNumber,
        keytable: FaultTolerantKeyTable,
        pin_counts: Vec<(Cid, u32)>,
    ) -> Self {
        Checkpoint {
            block_num,
            keytable,
            pin_counts,
        }
    }

    pub fn height(&self) -> BlockNumber {
        self.block_num
    }

    pub fn keytable(self) -> FaultTolerantKeyTable {
        self.keytable
    }

    pub fn pin_counts(&self) -> Vec<(Cid, u32)> {
        self.pin_counts.clone()
    }
}

pub struct DbCheckpoint {
    db: Db,
    rep_factor: u32,
    keytable_log: bool,
    node_id: NodeId,
}

impl DbCheckpoint {
    pub fn from_config(config: &Config) -> Self {
        // Open database
        let db = Self::open_db_from_node(config.node_id());
        Self {
            db,
            rep_factor: config.rep_factor,
            keytable_log: config.keytable_log,
            node_id: config.node_id(),
        }
    }

    pub fn from_values(rep_factor: u32, node_id: NodeId, keytable_log: bool) -> Self {
        // Open database
        let db = Self::open_db_from_node(node_id);
        Self {
            db,
            rep_factor,
            keytable_log,
            node_id,
        }
    }

    fn open_db_from_node(node_id: NodeId) -> Db {
        let home = std::env::var("HOME").unwrap();
        let node_id = hex::encode(&node_id.encode()[..=8]);
        let db_name = format!("{}/node_{}/db", home, node_id);
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
            FaultTolerantKeyTable::new(self.rep_factor, self.node_id, self.keytable_log);
        let mut pin_counts = Vec::new();
        for idx in 0..self.rep_factor {
            let key = format!("partition_{}", idx);
            let row = self.read_checkpoint_value::<TableRow>(&key)?;

            if let Some(row) = row {
                keytable.mutable_table().add_row(row.clone());

                // Read the pin counts
                for cid in row.values() {
                    let pin_count = self.read_cid_pin_count(cid)?;
                    pin_counts.push((cid.clone(), pin_count));
                }
            }
        }

        let block_num = self.read_blocknumber()?.unwrap_or_default();

        Ok(Checkpoint::new(block_num, keytable, pin_counts))
    }

    /// Commits to storage the block number that the node has processed in terms of events and the affected rows in the keytable.
    pub fn commit_checkpoint(
        &self,
        block_num: BlockNumber,
        rows: Vec<&TableRow>,
        pin_counts: Vec<(Cid, u32)>,
    ) -> Result<()> {
        let mut batch = DbBatch::default();
        batch.insert(BLOCK_NUM_KEY, block_num.encode());
        for (idx, row) in rows.iter().enumerate() {
            let key = format!("partition_{}", idx);
            batch.insert(key.as_bytes(), row.encode());
        }

        for (cid, pin_count) in pin_counts {
            if pin_count == 0 {
                batch.remove(cid.as_ref());
            } else {
                batch.insert(cid.as_ref(), pin_count.encode());
            }
        }

        // Commit the batch
        self.db.apply_batch(batch)?;

        Ok(())
    }

    pub fn read_blocknumber(&self) -> Result<Option<BlockNumber>> {
        let block_num = self.read_checkpoint_value::<BlockNumber>(BLOCK_NUM_KEY)?;

        Ok(block_num)
    }

    pub fn read_cid_pin_count(&self, cid: &Cid) -> Result<u32> {
        let pin_count = self
            .read_checkpoint_value::<u32>(cid.as_ref())?
            .ok_or(anyhow::anyhow!(
                "Cannot read pin count for cid: {:?} because it does not exist",
                cid
            ))?;

        Ok(pin_count)
    }
}

const BLOCK_NUM_KEY: &str = "block_num";
