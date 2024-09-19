use super::{
	dispatcher::{EventDispatcher, MutableDispatcher},
	NodeEvent,
};
use crate::{
	db::checkpointing::{BarrierCheckpoint, DbCheckpoint},
	ipfs::client::IpfsClient,
	substrate::client::SubstrateClient,
	types::channels::{self, PinningReadingHandles, PinningWritingHandles},
};
use anyhow::Result;
use primitives::BlockNumber;
use std::sync::Arc;
use tokio::task::JoinHandle;

// Maybe it needs a channel rather than a vector of capsule events
pub struct NodeEventsPool<'a> {
	/// Substrate client api
	client_api: Arc<SubstrateClient>,
	// Handles to reading channels (to recive the block number and events)
	reading_handles: PinningReadingHandles,
	// Handles to writing channels (to send the block number and events)
	writing_handles: PinningWritingHandles,
	// Checkpoint to recover events from a given block number
	checkpoint: Option<BarrierCheckpoint>,
	/// Event Dispatcher
	dispatcher: EventDispatcher<'a>,
	/// Events to be processed before listening the channel of upcoming events
	events: Vec<NodeEvent>,
}

impl<'a> NodeEventsPool<'a> {
	pub fn new(
		client_api: Arc<SubstrateClient>,
		ipfs: &'a mut IpfsClient,
		db: &'a DbCheckpoint,
	) -> Result<Self> {
		// Create handles to write and read from the channel
		let (writing_handles, reading_handles) = channels::build_channels();

		let checkpoint = db.read_barrier_checkpoint()?;

		Ok(Self {
			client_api,
			reading_handles,
			writing_handles,
			checkpoint,
			dispatcher: (db, ipfs),
			events: Vec::new(),
		})
	}

	/// Pulls new finalized capsule events from the chain and produces them into a channel.
	pub async fn produce_capsule_events(&mut self) -> Result<JoinHandle<Result<()>>> {
		// Clone the Arc to use it in the thread that handles the event subscription
		let client_api = Arc::clone(&self.client_api);
		// Clone the writing handles to use it in the spawned thread
		let mut writing_handles = self.writing_handles.clone();

		// Spawn a new task to subscribe to new capsule events.
		let subscription = tokio::spawn(async move {
			let mut blocks_sub = client_api.get_api().blocks().subscribe_finalized().await?;

			while let Some(block) = blocks_sub.next().await {
				let block = block?;
				let block_num = block.number() as BlockNumber;
				if !writing_handles.is_block_number_sent() {
					// Send the first block number to the channel so the main thread knows the upper bound for event recovery.
					writing_handles.send_block_number(block_num).await?;
				}

				let events = client_api.events_at(block.hash().into()).await?;
				for event in events {
					// Send the new events to the channel for processing.
					writing_handles.send_event(event)?;
				}
				// Send checkpointing event
				writing_handles.send_event(NodeEvent::BlockCheckpoint(block_num))?;
			}

			unreachable!("Unexpected chain behavior: block finalization has stopped. The chain is expected to continuously finalize blocks.");
		});

		// Receive the new finalized block number.
		let new_finalized_block = self.reading_handles.receive_block_number()?;

		// Check if we have a checkpoint and recover events if necessary.
		if let Some(checkpoint) = &self.checkpoint {
			// Recover events in the specified block range.
			let recover_events =
				self.client_api.events_in_range(checkpoint + 1, new_finalized_block - 1).await?;

			self.events.extend(recover_events);
		}

		Ok(subscription)
	}

	/// Consumes recieving events, first from the events `Vec` and then from the channel for new finalized events
	pub async fn consume_capsule_events(&mut self) -> Result<()> {
		// First, dispatch recovered events
		for event in &self.events {
			self.dispatcher.dispatch(event).await?;
		}
		self.events.clear();

		// Consume and dispatch upcoming events from the channel
		while let Some(event) = self.reading_handles.receive_events().await {
			self.dispatcher.dispatch(&event).await?;
		}

		Ok(())
	}
}
