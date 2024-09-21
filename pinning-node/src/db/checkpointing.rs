use anyhow::Result;
use async_trait::async_trait;
use codec::{Decode, Encode};
use primitives::BlockNumber;
use sled::Db;

use crate::utils::traits::Dispatcher;

pub type BarrierCheckpoint = BlockNumber;

pub struct DbCheckpoint(Db);

impl DbCheckpoint {
	pub fn new() -> Self {
		// Open database
		let db = sled::open("checkpointing_db").unwrap();
		Self(db)
	}

	/// Commits to storage the block number that the node has processed in terms of events.
	pub fn barrier_checkpoint(&self, checkpoint: BarrierCheckpoint) -> Result<()> {
		self.0
			.insert(b"checkpoint", checkpoint.encode())
			.map_err(|_| anyhow::anyhow!("Failed to insert checkpoint"))?;

		Ok(())
	}

	/// Retrieves the block number that the node has currently processed in terms of events.
	pub fn read_barrier_checkpoint(&self) -> Result<Option<BarrierCheckpoint>> {
		let checkpoint = self
			.0
			.get(b"checkpoint")
			.map_err(|_| anyhow::anyhow!("Failed to read checkpoint from db"))?;

		if let Some(checkpoint) = checkpoint {
			let checkpoint = BarrierCheckpoint::decode(&mut checkpoint.as_ref())
				.map_err(|_| anyhow::anyhow!("Failed to decode checkpoint"))?;

			Ok(Some(checkpoint))
		} else {
			Ok(None)
		}
	}
}

#[async_trait(?Send)]
impl Dispatcher<BarrierCheckpoint> for DbCheckpoint {
	async fn dispatch(&self, checkpoint: &BarrierCheckpoint) -> Result<()> {
		self.barrier_checkpoint(*checkpoint)
	}
}
