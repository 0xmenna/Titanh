use super::{
	channels::{self, PinningReadingHandles, PinningWritingHandles},
	pinning::{PinningCapsuleEvent, PinningEvent},
};
use crate::{db::checkpointing::DbCheckpoint, substrate::client::SubstrateClient};
use anyhow::Result;
use codec::{Decode, Encode};
use ipfs_api_backend_hyper::IpfsApi;
use ipfs_api_backend_hyper::IpfsClient;
use primitives::BlockNumber;
use std::sync::Arc;
use tokio::task;

#[derive(Encode, Decode)]
pub struct CheckpointEvents {
	pub block_number: BlockNumber,
	pub processed_events: u32,
}

// Maybe it needs a channel rather than a vector of capsule events
pub struct PinningEventsPool<'a> {
	/// Db to handle checkpoints
	db: &'a DbCheckpoint,
	/// Ipfs client for pinning specific operation
	ipfs: IpfsClient,
	/// Substrate client api
	client_api: Arc<SubstrateClient>,
	// Handles to reading channels (to recive the block number and events)
	reading_handles: PinningReadingHandles,
	// Handles to writing channels (to send the block number and events)
	writing_handles: PinningWritingHandles,
	// Checkpoint to recover events from a given block number
	checkpoint: Option<CheckpointEvents>,
	/// Events to be processed before listening the channel of upcoming events
	events: Vec<PinningCapsuleEvent>,
}

impl<'a> PinningEventsPool<'a> {
	pub fn new(client_api: Arc<SubstrateClient>, db: &DbCheckpoint) -> Result<Self> {
		let ipfs = IpfsClient::default();
		// Create handles to write and read from the channel
		let (writing_handles, reading_handles) = channels::build_channels();

		let checkpoint = db.read_checkpoint_for_events()?;

		Ok(Self {
			db,
			ipfs,
			client_api,
			reading_handles,
			writing_handles,
			checkpoint,
			events: Vec::new(),
		})
	}

	pub fn add_events(&mut self, events: Vec<PinningCapsuleEvent>) {
		self.events.extend(events);
	}

	/// Pulls new finalized capsule events from the chain and produces them into a channel.
	pub fn produce_capsule_events(&self) -> Result<()> {
		// Clone the Arc to use it in the thread that handles the event subscription
		let client_api = Arc::clone(&self.client_api);
		// Clone the writing handles to use it into the spawned thread
		let writing_handles = self.writing_handles.clone();

		// Spawn a new thread that subscribes to new capsule events.
		let subscription: task::JoinHandle<anyhow::Result<()>> = task::spawn(async move {
			let mut blocks_sub = client_api.get_api().blocks().subscribe_finalized().await?;

			while let Some(block) = blocks_sub.next().await {
				let block = block?;
				if !writing_handles.is_block_number_sent() {
					// Send through the writing block handle of the channel the first block number (of new finalized blocks) so that the main thread knows what is the upper bound block, used for events recovering.
					writing_handles.send_block_number(block.number() as BlockNumber).await?;
				}

				let events = client_api.pinning_events_at(block.hash().into()).await?;
				for event in events.into_iter() {
					// Senf the new events to process through the writing events handle of the cannel.
					writing_handles.send_event(event)?;
				}
			}

			unreachable!("Chain error, blocks must continuosly arrive.")
		});

		let block_number = self.reading_handles.receive_block_number()?;

		// Quando c'e il block il Receiver<PinningCapsuleEvents> fa la get_events() per quelli vecchi leggendoli dal canale in lettura degli eventi
		// agiunge eventi a self => self.events.extend(events);
		// termina

		Ok(())
	}

	/// Consumes recieving events, first from the events `Vec` and then from the channel for new finalized events
	pub fn consume_capsule_events(&self) {
		// prima processa tutti gli eventi in self.events
		// legge dal canale e porcessa eventi

		// cicla su events, sono per forza quelli vecchi perché quelli nuovi vegono messi nel canale
		// chiama la dispatch_event dell'evento e in base al tipo di evento esegue l'operazione,
		// ovvero uno switch
	}

	/// Dispatches a pinning event, by adding , updating or removing a pin with the IPFS Client
	pub fn dispatch_event(&self, event: &PinningCapsuleEvent) -> Result<()> {
		Ok(())
	}
}
