use anyhow::Result;
use codec::{Decode, Encode};
use primitives::BlockNumber;
use sled::Db;

pub struct DbCheckpoint(Db);

impl DbCheckpoint {
	pub fn new() -> Self {
		// Open database
		let db = sled::open("checkpointing_db").unwrap();
		Self(db)
	}

	pub fn checkpoint(&self, at: &BlockNumber) -> Result<()> {
		self.0
			.insert(b"checkpoint", at.encode())
			.map_err(|_| anyhow::anyhow!("Failed to insert checkpoint"))?;

		Ok(())
	}

	pub fn get_latest_checkpoint(&self) -> Result<Option<BlockNumber>> {
		let block = self
			.0
			.get(b"checkpoint")
			.map_err(|_| anyhow::anyhow!("Failed to read block number from db"))?;

		if let Some(block_number) = block {
			let block_number = BlockNumber::decode(&mut block_number.as_ref())
				.map_err(|_| anyhow::anyhow!("Failed to decode block number for checkpoint"))?;

			Ok(Some(block_number))
		} else {
			Ok(None)
		}
	}
}
