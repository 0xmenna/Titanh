use crate::{
	substrate::client::SubstrateClient,
	types::events_pool::NodeEventsPool,
	utils::ref_builder::{AtomicRef, MutableRef},
};
use anyhow::Result;
use api::common_types::{BlockInfo, BlockNumber};
use tokio::task::JoinHandle;

pub struct NodeProducer {
	/// Substrate client
	client: AtomicRef<SubstrateClient>,
	/// Pool of events
	events_pool: MutableRef<NodeEventsPool>,
	// The startup block number, used to recover events
	startup_block: BlockNumber,
}

impl NodeProducer {
	pub fn new(
		client: AtomicRef<SubstrateClient>,
		events_pool: MutableRef<NodeEventsPool>,
		startup_block: BlockNumber,
	) -> Self {
		Self { client, events_pool, startup_block }
	}

	/// Pulls new finalized events from the chain and produces them into the events pool (through a channel).
	pub async fn produce_events(&mut self) -> Result<JoinHandle<Result<()>>> {
		// Clone the Arc to use it in the thread that handles the event subscription
		let client_api = self.client.clone();

		let mut events_pool = self.events_pool.borrow_mut();

		// Clone the writing handles to use it in the spawned thread
		let mut pool_write_handle = events_pool.write_handle();
		// Spawn a new task to subscribe to new capsule events.
		let subscription = tokio::spawn(async move {
			let mut blocks_sub =
				client_api.api().substrate_api.blocks().subscribe_finalized().await?;

			while let Some(block) = blocks_sub.next().await {
				let block = block?;
				let block_num = block.number();
				if !pool_write_handle.is_block_number_sent() {
					// Send the first block number to the channel so the main thread knows the upper bound for event recovery.
					pool_write_handle.send_block_number(block_num).await?;
				}
				let block = BlockInfo::new(block_num, block.hash().into());
				let events = client_api.events_at(block).await?;
				for event in events {
					// Send the new events to the channel for processing.
					pool_write_handle.send_event(event)?;
				}
			}

			unreachable!("Unexpected chain behavior: block finalization has stopped. The chain is expected to continuously finalize blocks.");
		});

		// Receive the new finalized block number.
		let new_finalized_block = events_pool.read_handle().receive_block_number()?;

		// Recover events in the specified block range, if any.
		let recover_batch = self
			.client
			.events_in_range(self.startup_block + 1, new_finalized_block - 1)
			.await?;

		events_pool.insert_batch(recover_batch);

		Ok(subscription)
	}
}
