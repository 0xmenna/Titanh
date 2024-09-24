use anyhow::Result;
use api::common_types::BlockNumber;
use async_trait::async_trait;
use codec::{Decode, Encode};
use sled::Db;

use crate::{
	types::{events::BlockNumberEvent, keytable::KeyMap},
	utils::traits::Dispatcher,
};

pub type CapsulesCheckpoint = BlockNumber;
pub type KeyMapCheckpoint = KeyMap;

pub struct Checkpoint {
	/// The capsules checkpoint. Holds the block number until which the node has processed capsule events.
	pub capsules: CapsulesCheckpoint,
	/// Checkpoint of the keymap managed by the pinning node.
	pub keymap: KeyMapCheckpoint,
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
		let capsules_checkpoint = self.read_capsules_checkpoint()?;
		let keymap_checkpoint = self.read_keymap_checkpoint()?;

		Ok(Checkpoint {
			capsules: capsules_checkpoint.unwrap_or(0),
			keymap: keymap_checkpoint.unwrap_or_default(),
		})
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
	pub fn capsules_checkpoint(&self, checkpoint: &CapsulesCheckpoint) -> Result<()> {
		self.checkpoint("pinning_checkpoint", checkpoint)
	}

	/// Retrieves the block number that the node has currently processed in terms of events.
	pub fn read_capsules_checkpoint(&self) -> Result<Option<CapsulesCheckpoint>> {
		let checkpoint = self.read_checkpoint::<CapsulesCheckpoint>("pinning_checkpoint")?;

		Ok(checkpoint)
	}

	/// Commits to storage the keymap managed by the pinning node.
	pub fn keymap_checkpoint(&self, checkpoint: &KeyMapCheckpoint) -> Result<()> {
		self.checkpoint("keymap_checkpoint", checkpoint)
	}

	/// Retrieves the keymap managed by the pinning node.
	pub fn read_keymap_checkpoint(&self) -> Result<Option<KeyMapCheckpoint>> {
		let checkpoint = self.read_checkpoint::<KeyMapCheckpoint>("keymap_checkpoint")?;

		Ok(checkpoint)
	}
}

#[async_trait(?Send)]
impl Dispatcher<BlockNumberEvent> for DbCheckpoint {
	async fn dispatch(&self, checkpoint: &CapsulesCheckpoint) -> Result<()> {
		self.capsules_checkpoint(checkpoint)
	}
}

// TODO: maybe modify this
#[async_trait(?Send)]
impl Dispatcher<KeyMapCheckpoint> for DbCheckpoint {
	async fn dispatch(&self, checkpoint: &KeyMapCheckpoint) -> Result<()> {
		self.keymap_checkpoint(checkpoint)
	}
}
