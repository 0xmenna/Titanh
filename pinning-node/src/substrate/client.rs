use crate::{
	types::{
		events::{self, EventType, NodeEvent, RingUpdateEvent, TitanhEvent},
		keytable::{FaultTolerantBTreeMap, KeyMap},
	},
	utils::{
		ref_builder::{AtomicRef, MutableRef},
		traits::MutableDispatcher,
	},
};
use anyhow::Result;
use api::{
	common_types::{BlockHash, BlockInfo, BlockNumber},
	pinning_committee_types::{NodeId, PinningRing},
	titanh, TitanhApi,
};
use async_trait::async_trait;
use codec::Decode;

pub struct SubstratePinningClient {
	api: TitanhApi,
	/// The node id bounded to the client
	node_id: NodeId,
	/// A reference to the pinning ring
	ring: MutableRef<PinningRing>,
}

impl SubstratePinningClient {
	pub fn new(api: TitanhApi, node_id: NodeId, ring: MutableRef<PinningRing>) -> Self {
		SubstratePinningClient { api, node_id, ring }
	}

	/// Given a block hash, it returns the list of events that are relevant to the pinning node, based on the pinning ring.
	pub async fn events_at(&mut self, block: BlockInfo) -> Result<Vec<NodeEvent>> {
		let events_query = titanh::storage().system().events();
		// Events at block identified by `block_hash`
		let runtime_events = self.api.query(&events_query, Some(block.hash)).await?;

		let mut events = Vec::new();
		for event_record in runtime_events.into_iter() {
			let event = events::try_event_from_runtime(event_record.event);

			if let Some(event) = event {
				match event {
					TitanhEvent::Capsules(capsule_evenet) => {
						// If the pinning node is responsible for the key, then we get the partition number to which the key belongs
						let maybe_partition = self
							.ring
							.borrow()
							.key_node_partition(capsule_evenet.key, self.node_id)?;

						if let Some(partition) = maybe_partition {
							let pin_event = NodeEvent::pinning_event(partition, capsule_evenet);
							events.push(pin_event);
						}
					},
					TitanhEvent::PinningCommittee(ring_event) => match ring_event {
						RingUpdateEvent::NewPinningNode(node_id) => {
							let update = self.ring.borrow_mut().insert_node(node_id.clone())?;

							let key_range = update.node_range(&self.node_id);
							if let Some(range) = key_range {
								events.push(range.into());
							}
						},
						RingUpdateEvent::RemovePinningNode { node_id, db_keys } => {
							let update = self.ring.borrow_mut().remove_node(node_id.clone())?;

							let key_range = update.node_range(&self.node_id);

							if let Some(_) = key_range {
								events.push(NodeEvent::TransferKeys(db_keys));
								let transferrred_map =
									FaultTolerantBTreeMap::decode(&mut db_keys.as_ref())?;
								// recover events of new uncovered blocks
								let from_block = transferrred_map.at();
								let to_block = block.number;
							}
						},
					},
					_ => Err(anyhow::anyhow!("Unsupported event type"))?,
				};
			}
		}
		Ok(events)
	}

	/// Returns the list of events of type `EventType` occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
	pub async fn events_in_range(
		&self,
		start: BlockNumber,
		end: BlockNumber,
		event: EventType,
	) -> Result<Vec<NodeEvent>> {
		let mut events = Vec::new();

		for block_number in start..=end {
			let block_hash = self.api.block_hash(block_number).await?;

			let node_events = self.events_at(block_hash, event.clone()).await?;
			events.extend(node_events);
			// Handle different types of checkpointing control events
			if event == EventType::Capsules {
				// Add capsules barrier event for later checkpointing
				events.push(NodeEvent::CapsulesBarrier(block_number));
			}
		}

		// We do this at the end because checkpointing the keymap is more expensive
		if event == EventType::PinningCommittee {
			// Add keymap barrier event for later checkpointing
			events.push(NodeEvent::KeyMapBarrier(end));
		}

		Ok(events)
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
