use crate::{
	types::events::{self, NodeEvent},
	utils::ref_builder::AtomicRef,
};
use anyhow::Result;
use api::{
	common_types::{BlockHash, BlockNumber},
	pinning_committee_types::{NodeId, PinningRing},
	titanh, TitanhApi,
};

pub struct SubstratePinningClient {
	api: TitanhApi,
	/// The node id bounded to the client
	node_id: NodeId,
	/// A reference to the pinning ring
	pinning_ring: AtomicRef<PinningRing>,
}

impl SubstratePinningClient {
	pub fn new(
		api: TitanhApi,
		// The node id bounded to the client
		node_id: NodeId,
		// A reference to the pinning ring
		pinning_ring: AtomicRef<PinningRing>,
	) -> Self {
		SubstratePinningClient { api, node_id, pinning_ring }
	}

	/// Given a block hash, it returns the list of events that are relevant to the pinning node, based on the pinning ring.
	pub async fn events_at(&self, block_hash: BlockHash) -> Result<Vec<NodeEvent>> {
		let events_query = titanh::storage().system().events();
		// Events at block identified by `block_hash`
		let events = self.api.query(&events_query, Some(block_hash)).await?;

		let mut pinning_events = Vec::new();
		for event_record in events.into_iter() {
			let event = events::try_pinning_event_from_runtime(event_record.event);

			if let Some(event) = event {
				let is_node_replica =
					self.pinning_ring.is_key_owned_by_node(event.key, self.node_id)?;
				if is_node_replica {
					pinning_events.push(event.into())
				}
			}
		}
		Ok(pinning_events)
	}

	/// Returns the list of pinning capsule events occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
	pub async fn events_in_range(
		&self,
		start: BlockNumber,
		end: BlockNumber,
	) -> Result<Vec<NodeEvent>> {
		let mut capsule_events = Vec::new();
		for block_number in start..=end {
			let block_hash = self.api.block_hash(block_number).await?;

			let events = self.events_at(block_hash).await?;
			capsule_events.extend(events);
			// Add barrier event for later checkpointing
			capsule_events.push(NodeEvent::BlockCheckpoint(block_number));
		}

		Ok(capsule_events)
	}

	pub fn api(&self) -> &TitanhApi {
		&self.api
	}

	pub fn ring(&self) -> AtomicRef<PinningRing> {
		self.pinning_ring.clone()
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
}
