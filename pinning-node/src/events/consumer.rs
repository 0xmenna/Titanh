use super::dispatcher::{traits::AsyncMutableDispatcher, NodeEventDispatcher};
use crate::{
    types::{batch::Batch, events_pool::NodeEventsPool},
    utils::ref_builder::MutableRef,
};
use anyhow::Result;
use std::mem;

pub struct NodeConsumer {
    /// Pool of events
    events_pool: MutableRef<NodeEventsPool>,
    /// Event dispatcher
    dispatcher: NodeEventDispatcher,
}

impl NodeConsumer {
    pub fn new(events_pool: MutableRef<NodeEventsPool>, dispatcher: NodeEventDispatcher) -> Self {
        Self {
            events_pool,
            dispatcher,
        }
    }

    /// Consumes recieving events, first from the events `Vec` and then from the channel for new finalized events
    pub async fn consume_events(mut self) -> Result<()> {
        let mut events_pool = self.events_pool.borrow_mut();

        // Consume and dispatch upcoming events from the pool (aka channel), grouping them into batches.
        let mut consuming_batch = Batch::default();
        while let Some(event) = events_pool.read_handle().receive_events().await {
            consuming_batch.insert(event.clone());
            log::info!("Consuming event {:?}, added to current batch", event);

            if let Some(block_num) = event.block_barrier_event() {
                log::info!("Consuming batch identified by block number: {}", block_num);
                // Dispatch the batch
                let dispatchable_batch = mem::take(&mut consuming_batch);
                self.dispatcher.async_dispatch(dispatchable_batch).await?;
                log::info!(
                    "Dispatching of batch with size: {} completed",
                    consuming_batch.size()
                );
            }
        }

        Ok(())
    }
}
