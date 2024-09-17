use std::sync::mpsc;

use anyhow::Result;
use codec::{Decode, Encode};
use primitives::BlockNumber;
use sled::Db;

use crate::types::events::CheckpointEvents;

pub struct DbCheckpoint(Db);

impl DbCheckpoint {
	pub fn new() -> Self {
		// Open database
		let db = sled::open("checkpointing_db").unwrap();
		Self(db)
	}

	pub fn checkpoint_for_events(&self, checkpoint: CheckpointEvents) -> Result<()> {
		self.0
			.insert(b"checkpoint", checkpoint.encode())
			.map_err(|_| anyhow::anyhow!("Failed to insert checkpoint"))?;

		Ok(())
	}

	pub fn read_checkpoint_for_events(&self) -> Result<Option<CheckpointEvents>> {
		let checkpoint = self
			.0
			.get(b"checkpoint")
			.map_err(|_| anyhow::anyhow!("Failed to read checkpoint from db"))?;

		if let Some(checkpoint) = checkpoint {
			let checkpoint = CheckpointEvents::decode(&mut checkpoint.as_ref())
				.map_err(|_| anyhow::anyhow!("Failed to decode checkpoint"))?;

			Ok(Some(checkpoint))
		} else {
			Ok(None)
		}
	}
}
