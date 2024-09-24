use super::{dispatcher::EventDispatcher, NodeEvent};
use crate::{
	db::checkpointing::{Checkpoint, DbCheckpoint},
	ipfs::client::IpfsClient,
	substrate::client::SubstratePinningClient,
	types::{
		channels::{self, PinningReadingHandles, PinningWritingHandles},
		events::EventType,
	},
	utils::{
		ref_builder::{AtomicRef, MutableRef, Ref},
		traits::MutableDispatcher,
	},
};
use anyhow::Result;
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
	/// Events to be processed before listening the channel of upcoming events
	events: Vec<NodeEvent>,
}

impl NodeEventsPool {
	pub fn new(
		client_api: AtomicRef<SubstratePinningClient>,
		db: Ref<DbCheckpoint>,
		ipfs: MutableRef<IpfsClient>,
	) -> Self {
		// Create handles to write and read from the channel
		let (writing_handles, reading_handles) = channels::build_channels();

		let checkpoint = db.read_all().expect("Failed to read checkpoint");

		Self {
			client_api,
			reading_handles,
			writing_handles,
			checkpoint,
			dispatcher: (db, ipfs),
			events: Vec::new(),
		}
	}

	/// Pulls new finalized events of type `EventType` from the chain and produces them into a channel.
	pub async fn produce_events(&mut self, event: EventType) -> Result<JoinHandle<Result<()>>> {
		// Clone the Arc to use it in the thread that handles the event subscription
		let client_api = self.client_api.clone();

		// Clone the writing handles to use it in the spawned thread
		let mut writing_handles = self.writing_handles.clone();
		let event_type = event.clone();
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

				let events = client_api.events_at(block.hash().into(), event_type.clone()).await?;
				for event in events {
					// Send the new events to the channel for processing.
					writing_handles.send_event(event)?;
				}
				// Handle the checkpointing for the different events
				match event_type {
					EventType::Capsules => {
						// Send checkpointing event at every block
						writing_handles.send_event(NodeEvent::CapsulesBarrier(block_num))?;
					},
					EventType::PinningCommittee => {
						if block_num % KEYMAP_CHECKPOINT_PERIOD == 0 {
							writing_handles.send_event(NodeEvent::KeyMapBarrier(block_num))?;
						}
					},
				}
			}

			unreachable!("Unexpected chain behavior: block finalization has stopped. The chain is expected to continuously finalize blocks.");
		});

		// Receive the new finalized block number.
		let new_finalized_block = self.reading_handles.receive_block_number()?;

		// Recover events in the specified block range, if any.
		let checkpoint = match event {
			EventType::Capsules => self.checkpoint.capsules,
			EventType::PinningCommittee => self.checkpoint.keymap.at(),
		};
		let recover_events = self
			.client_api
			.events_in_range(checkpoint + 1, new_finalized_block - 1, event)
			.await?;

		self.events.extend(recover_events);

		Ok(subscription)
	}

	/// Consumes recieving events, first from the events `Vec` and then from the channel for new finalized events
	pub async fn consume_events(&mut self) -> Result<()> {
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
