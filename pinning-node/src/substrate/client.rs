use crate::{
    types::{
        batch::Batch,
        events::{self, NodeEvent},
    },
    utils::ref_builder::{self, AtomicRef},
};
use anyhow::Result;
use api::{
    common_types::{BlockInfo, BlockNumber},
    pinning_committee_types::{NodeId, PinningRing},
    TitanhApi,
};

pub struct SubstrateClient {
    api: TitanhApi,
    /// The node id bounded to the client
    node_id: NodeId,
    /// The block at which the client has been intialized
    block: BlockInfo,
}

impl SubstrateClient {
    pub fn new(api: TitanhApi, node_id: NodeId, block: BlockInfo) -> Self {
        Self {
            api,
            node_id,
            block,
        }
    }

    /// Return the events of a given block, in a batch.
    pub async fn events_at(&self, block: BlockInfo) -> Result<Batch<NodeEvent>> {
        // Events at block identified by `block_hash`
        let runtime_events = self.api.runtime_events(Some(block.hash)).await?;

        let mut batch = Batch::default();
        for event_record in runtime_events.into_iter() {
            let node_event = events::try_event_from_runtime(event_record.event);
            if let Some(event) = node_event {
                if event.is_committee_event() && self.block.number >= block.number {
                    // The client has been initialized at a later block, so we ignore if the event is a committee event (join or leave). This is because the node boudend to the client has a most up to date ring.
                    continue;
                }
                batch.insert(event);
            }
        }

        // Add a block barrier event for later checkpointing
        batch.insert(NodeEvent::BlockBarrier(block.number));

        Ok(batch)
    }

    /// Returns the list of events occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
    pub async fn events_in_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> Result<Batch<NodeEvent>> {
        let mut batch = Batch::default();

        for block_number in start..=end {
            let block = BlockInfo {
                number: block_number,
                hash: self.api.block_hash(block_number).await?,
            };

            let block_batch = self.events_at(block).await?;
            batch.extend(block_batch);
        }

        Ok(batch)
    }

    pub fn api(&self) -> &TitanhApi {
        &self.api
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub async fn ring(&self) -> PinningRing {
        let ring = self
            .api
            .pinning_committee()
            .pinning_ring_at(self.block.hash)
            .await
            .expect("Pinning ring is expected to be initialized");

        ring
    }

    pub fn arc(self) -> AtomicRef<Self> {
        ref_builder::create_atomic_ref(self)
    }

    pub fn height(&self) -> BlockNumber {
        self.block.number
    }
}
