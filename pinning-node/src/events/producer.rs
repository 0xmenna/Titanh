use crate::{
    substrate::client::SubstrateClient,
    types::{channels::PoolWritingHandle, events::NodeEvent, events_pool::NodeEventsPool},
    utils::{
        self,
        ref_builder::{AtomicRef, MutableRef},
    },
};
use anyhow::Result;
use api::{
    common_types::{BlockInfo, BlockNumber},
    titanh::{self},
};
use tokio::task::JoinHandle;

pub struct NodeProducer {
    /// Substrate client
    client: AtomicRef<SubstrateClient>,
    /// Pool of events
    events_pool: MutableRef<NodeEventsPool>,
    /// The startup block number, used to produce events for recovering
    start_block_recovering: BlockNumber,
    /// The block number at which the ring is up to date.
    ring_height: BlockNumber,
}

impl NodeProducer {
    pub fn new(
        client: AtomicRef<SubstrateClient>,
        events_pool: MutableRef<NodeEventsPool>,
        start_block_recovering: BlockNumber,
        ring_height: BlockNumber,
    ) -> Self {
        Self {
            client,
            events_pool,
            start_block_recovering,
            ring_height,
        }
    }

    /// Spawns a thread that procudes events into the events pool (through a channel), by first recovering old events and then pulling new events in real-time from the chain's finalized blocks.
    pub fn produce_events(&mut self) -> JoinHandle<Result<()>> {
        // Clone the Arc to use it in the thread
        let client = self.client.clone();

        // Clone the writing handles to use it in the spawned thread
        let mut pool_write_handle = self.events_pool.borrow_mut().write_handle();
        // Block number from which event recovery should start
        let start_block_recovering = self.start_block_recovering;
        let ring_height = self.ring_height;
        // Spawn a new task
        tokio::spawn(async move {
            // Subscribe to finalized blocks to get events in real-time
            let mut blocks_sub = client
                .api()
                .substrate_api
                .blocks()
                .subscribe_finalized()
                .await?;

            let mut has_recovered = false;
            while let Some(block) = blocks_sub.next().await {
                let block = block?;
                let block_num = block.number();

                if !has_recovered {
                    // Before processing new events of finalized blocks, we must recover the events.

                    log::info!(
                        "Starting to recover events from {} to {}",
                        start_block_recovering,
                        block_num.saturating_sub(1)
                    );
                    produce_recover_events(
                        &client,
                        &mut pool_write_handle,
                        start_block_recovering,
                        block_num,
                        ring_height,
                    )
                    .await?;
                    log::info!("Recover events produced successfully");

                    has_recovered = true;
                }

                let block = BlockInfo::new(block_num, block.hash().into());
                let events = client.events_at(block).await?;
                for event in events {
                    // Send the new events to the channel for processing.
                    pool_write_handle.send_event(event.clone())?;
                    log::info!("Produced new event: {:?}", event);
                }
            }

            unreachable!("Unexpected chain behavior: block finalization has stopped. The chain is expected to continuously finalize blocks.");
        })
    }
}

/// Produce events to recover.
// Events can be recovered due to 2 possible scenarios:
// 1. The node has just started and must recover all capsules for producing pinning events.
// 2. The node has restarted and must recover the events from the last checkpointed block, which is the startup block if greater than 0.
pub async fn produce_recover_events(
    client: &SubstrateClient,
    writing_handle: &mut PoolWritingHandle,
    start_block_recovering: BlockNumber,
    lastest_finalized_block: BlockNumber,
    ring_height: BlockNumber,
) -> Result<()> {
    println!("ring_height: {}", ring_height);
    assert!(
        start_block_recovering <= ring_height && ring_height <= lastest_finalized_block,
        "Block ranges are invalid for event recovery"
    );

    let api = client.api();

    if start_block_recovering == 1 {
        // The node has just started and must recover all capsules for producing pinning events.
        // Since the ring of the pinning node is up to date until `ring_height`, we can only recover capsules until that height. From `ring_height + 1` we need to recover events from the remaining blocks (to spot eventual node removals or joins).

        // Recover capsules
        let block_num = if lastest_finalized_block == ring_height {
            lastest_finalized_block.saturating_sub(1)
        } else {
            ring_height
        };

        let block_hash = api.block_hash(block_num).await?;
        let storage = api.substrate_api.storage().at(block_hash);

        let capsules_query = titanh::storage().capsules().capsules_iter();
        let mut capsules_iter = storage.iter(capsules_query).await?;

        while let Some(Ok(kv)) = capsules_iter.next().await {
            let capsule = kv.value;
            let app_id = capsule.app_data.app_id;
            let metadata = capsule.app_data.data.0 .0;

            let capsule_id = utils::capsules::compute_capsule_id(metadata, app_id);
            let cid = capsule.cid.0 .0;

            let event = NodeEvent::from_capsule(capsule_id, cid)?;
            // Produce a pinning event
            writing_handle.send_event(event.clone())?;
            log::info!(
                "Produced a recover pinning event for node startup: {:?}",
                event
            );
        }

        // Recover events from the remaining blocks
        let events_after_ring_height = client
            .events_in_range(block_num + 1, lastest_finalized_block.saturating_sub(1))
            .await?;
        for event in events_after_ring_height {
            writing_handle.send_event(event.clone())?;
            log::info!("Produced a recover event for node startup: {:?}", event);
        }
    } else {
        // The node has restarted and must recover the events from the last checkpointed block
        let recover_batch = client
            .events_in_range(
                start_block_recovering,
                lastest_finalized_block.saturating_sub(1),
            )
            .await?;
        for event in recover_batch {
            writing_handle.send_event(event.clone())?;
            log::info!("Produced a recover event for node restart: {:?}", event);
        }
    }

    Ok(())
}
