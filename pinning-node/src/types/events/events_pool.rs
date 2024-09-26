use std::mem;

use super::{batch::Batch, dispatcher::EventDispatcher, NodeEvent};
use crate::{
	db::checkpointing::{Checkpoint, DbCheckpoint},
	ipfs::client::IpfsClient,
	substrate::client2::SubstratePinningClient,
	types::channels::{self, PinningReadingHandles, PinningWritingHandles},
	utils::{
		ref_builder::{AtomicRef, MutableRef, Ref},
		traits::MutableDispatcher,
	},
};
use anyhow::Result;
use api::common_types::BlockInfo;
use tokio::task::JoinHandle;

// Maybe it needs a channel rather than a vector of capsule events
pub struct NodeEventsPool {
	/// Substrate pinning client api
	client_api: AtomicRef<SubstratePinningClient>,
	// Handles to reading channels (to recive the block number and events)
	reading_handles: PinningReadingHandles,
	// Handles to writing channels (to send the block number and events)
	writing_handles: PinningWritingHandles,
	// Checkpoint to recover events from a given block number
	checkpoint: Checkpoint,
	/// Event Dispatcher
	dispatcher: EventDispatcher,
	/// Recovered events to be processed before listening the channel of upcoming events
	recovered_batch: Batch<NodeEvent>,
}

impl NodeEventsPool {
	pub fn new(
		client_api: AtomicRef<SubstratePinningClient>,
		db: Ref<DbCheckpoint>,
		ipfs: MutableRef<IpfsClient>,
	) -> Self {
		// Create handles to write and read from the channel
		let (writing_handles, reading_handles) = channels::build_channels();

		let checkpoint = db.get_checkpoint().expect("Failed to read checkpoint");

		Self {
			client_api,
			reading_handles,
			writing_handles,
			checkpoint,
			dispatcher: (db, ipfs),
			recovered_batch: Batch::default(),
		}
	}

	/// Pulls new finalized events from the chain and produces them into a channel.
	pub async fn produce_events(&mut self) -> Result<JoinHandle<Result<()>>> {
		// Clone the Arc to use it in the thread that handles the event subscription
		let client_api = self.client_api.clone();

		// Clone the writing handles to use it in the spawned thread
		let mut writing_handles = self.writing_handles.clone();
		// Spawn a new task to subscribe to new capsule events.
		let subscription = tokio::spawn(async move {
			let mut blocks_sub =
				client_api.api().substrate_api.blocks().subscribe_finalized().await?;

			// Used only for `EventType::PinningCommittee, i.e. when there is a ring update
			const KEYMAP_CHECKPOINT_PERIOD: u32 = 1200;

			while let Some(block) = blocks_sub.next().await {
				let block = block?;
				let block_num = block.number();
				if !writing_handles.is_block_number_sent() {
					// Send the first block number to the channel so the main thread knows the upper bound for event recovery.
					writing_handles.send_block_number(block_num).await?;
				}
				let block = BlockInfo::new(block_num, block.hash().into());
				let events = client_api.events_at(block).await?;
				for event in events {
					// Send the new events to the channel for processing.
					writing_handles.send_event(event)?;
				}
			}

			unreachable!("Unexpected chain behavior: block finalization has stopped. The chain is expected to continuously finalize blocks.");
		});

		// Receive the new finalized block number.
		let new_finalized_block = self.reading_handles.receive_block_number()?;

		// Recover events in the specified block range, if any.
		if let Some(block_num) = self.checkpoint.at() {
			let recover_batch =
				self.client_api.events_in_range(block_num + 1, new_finalized_block - 1).await?;

			self.recovered_batch.extend(recover_batch);
		}

		Ok(subscription)
	}

	/// Consumes recieving events, first from the events `Vec` and then from the channel for new finalized events
	pub async fn consume_events(mut self) -> Result<()> {
		// First, we dispatch the recovered batch of events
		self.dispatcher.dispatch(self.recovered_batch).await?;

		// Consume and dispatch upcoming events from the channel, grouping them into batches.
		let mut consuming_batch = Batch::default();
		while let Some(event) = self.reading_handles.receive_events().await {
			consuming_batch.insert(event.clone());
			if let Some(_) = event.block_barrier_event() {
				// Dispatch the batch
				let dispatchable_batch = mem::take(&mut consuming_batch);
				self.dispatcher.dispatch(dispatchable_batch).await?;
			}
		}

		Ok(())
	}
}
