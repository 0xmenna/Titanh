use crate::{
	types::{
		cid::Cid,
		keytable::{FaultTolerantKeyTable, KeyTable},
	},
	utils::traits::Dispatcher,
};
use anyhow::Result;
use api::{capsules_types::CapsuleKey, common_types::BlockNumber};
use async_trait::async_trait;
use codec::{Decode, Encode};
use sled::Db;

#[derive(Default)]
pub struct Checkpoint {
	/// The block number checkpoint. Holds the block number until which the node has processed events.
	pub block_num: Option<BlockNumber>,
	/// Checkpoint of the keymap managed by the pinning node.
	pub keytable: Option<FaultTolerantKeyTable>,
}

pub struct DbCheckpoint(Db);

impl DbCheckpoint {
	pub fn new() -> Self {
		// Open database
		let db = sled::open("checkpointing_db").unwrap();
		Self(db)
	}

	/// Reads all the checkpoints from the database.
	pub fn read_all(&self) -> Result<Checkpoint> {
		let block_num = self.read_blocknumber_checkpoint()?;
		let keytable = self.read_keytable_checkpoint()?;

		Ok(Checkpoint { block_num, keytable })
	}

	/// Commits to storage some key value pair that identifies some state of the node.
	fn checkpoint<C: Encode>(&self, key: &str, checkpoint: &C) -> Result<()> {
		self.0
			.insert(key.as_bytes(), checkpoint.encode())
			.map_err(|_| anyhow::anyhow!("Failed to insert checkpoint"))?;

		Ok(())
	}

	/// Retrieves some checkpoint from the database.
	fn read_checkpoint<D: Decode>(&self, key: &str) -> Result<Option<D>> {
		let checkpoint = self
			.0
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

	/// Commits to storage the block number that the node has processed in terms of events.
	pub fn blocknumber_checkpoint(&self, checkpoint: &BlockNumber) -> Result<()> {
		self.checkpoint("capsules_checkpoint", checkpoint)
	}

	/// Retrieves the block number that the node has currently processed in terms of events.
	pub fn read_blocknumber_checkpoint(&self) -> Result<Option<BlockNumber>> {
		let checkpoint = self.read_checkpoint::<BlockNumber>("capsules_checkpoint")?;

		Ok(checkpoint)
	}

	/// Commits to storage the keymap managed by the pinning node.
	pub fn keytable_checkpoint(&self, checkpoint: &FaultTolerantKeyTable) -> Result<()> {
		self.checkpoint("keytable_checkpoint", checkpoint)
	}

	/// Retrieves the keymap managed by the pinning node.
	pub fn read_keytable_checkpoint(&self) -> Result<Option<FaultTolerantKeyTable>> {
		let checkpoint = self.read_checkpoint::<FaultTolerantKeyTable>("keytable_checkpoint")?;

		Ok(checkpoint)
	}
}

type BlockNumberEvent = BlockNumber;
type KetTableEvent = FaultTolerantKeyTable;

#[async_trait(?Send)]
impl Dispatcher<BlockNumberEvent> for DbCheckpoint {
	async fn dispatch(&self, event: &BlockNumberEvent) -> Result<()> {
		self.blocknumber_checkpoint(event)
	}
}

// TODO: maybe modify this
#[async_trait(?Send)]
impl Dispatcher<KetTableEvent> for DbCheckpoint {
	async fn dispatch(&self, event: &KetTableEvent) -> Result<()> {
		self.keytable_checkpoint(event)
	}
}
