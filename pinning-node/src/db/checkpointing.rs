use crate::{
	types::{
		cid::Cid,
		keytable::{FaultTolerantKeyTable, OrderedRows, Row},
	},
	utils::traits::Dispatcher,
};
use anyhow::Result;
use api::{capsules_types::CapsuleKey, common_types::BlockNumber};
use async_trait::async_trait;
use codec::{Decode, Encode};
use sled::{Batch as DbBatch, Db};

#[derive(Encode, Decode, Clone)]
pub struct Checkpoint {
	/// The block number checkpoint. Holds the block number until which the node has processed events.
	pub block_num: Option<BlockNumber>,
	/// The keytable managed by the pinning node, up to date with the block number.
	pub keytable: FaultTolerantKeyTable,
}

impl Checkpoint {
	pub fn new(block_num: Option<BlockNumber>, keytable: FaultTolerantKeyTable) -> Self {
		Checkpoint { block_num, keytable }
	}

	pub fn at(&self) -> Option<BlockNumber> {
		self.block_num
	}
}

pub struct DbCheckpoint {
	db: Db,
	rep_factor: u32,
}

impl DbCheckpoint {
	pub fn new(rep_factor: u32) -> Self {
		// Open database
		let db = Self::open_db();
		Self { db, rep_factor }
	}

	fn open_db() -> Db {
		sled::open("checkpointing_db").unwrap()
	}

	// This is unbounded to the struct instance because we still don't know the `rep_factor`, since it's a value fetched remotely, based on the block number, this is why we read this first.
	pub fn get_blocknumber() -> Option<BlockNumber> {
		let db = Self::open_db();
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

	/// Commits to storage some value associated with a key.
	fn checkpoint_value<C: Encode>(db: &Db, key: &str, value: &C) -> Result<()> {
		db.insert(key.as_bytes(), value.encode())
			.map_err(|_| anyhow::anyhow!("Failed to insert checkpoint"))?;

		Ok(())
	}

	/// Retrieves the checkpoint.
	pub fn get_checkpoint(&self) -> Result<Checkpoint> {
		// Build the keytable
		let mut keytable = FaultTolerantKeyTable::new(self.rep_factor);
		for idx in 0..self.rep_factor {
			let key = format!("partition_{}", idx);
			let row = Self::read_checkpoint_value::<Row<CapsuleKey, Cid>>(&self.db, &key)?;

			if let Some(row) = row {
				keytable.add_row(row);
			}
		}

		let block_num = Self::read_blocknumber(&self.db)?;

		Ok(Checkpoint::new(block_num, keytable))
	}

	/// Commits to storage the block number that the node has processed in terms of events and the affected rows in the keytable.
	pub fn commit_checkpoint(
		&self,
		block_num: BlockNumber,
		rows: Vec<&Row<CapsuleKey, Cid>>,
	) -> Result<()> {
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

	pub fn rep_factor(&self) -> u32 {
		self.rep_factor
	}
}

pub struct CheckpointEvent<'a> {
	pub block_num: BlockNumber,
	/// checkpoint the keytable rows updated at the given block.
	pub table_rows: Vec<&'a Row<CapsuleKey, Cid>>,
}

impl CheckpointEvent<'_> {
	pub fn new(block_num: BlockNumber, table_rows: Vec<&Row<CapsuleKey, Cid>>) -> Self {
		CheckpointEvent { block_num, table_rows }
	}
}

#[async_trait(?Send)]
impl Dispatcher<CheckpointEvent<'_>, ()> for DbCheckpoint {
	async fn dispatch(&self, event: CheckpointEvent) -> Result<()> {
		let block_num = event.block_num;
		let rows = event.table_rows;
		self.commit_checkpoint(block_num, rows)?;

		Ok(())
	}
}
