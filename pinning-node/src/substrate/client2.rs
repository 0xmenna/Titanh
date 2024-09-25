use crate::{
	types::events::{self, KeyedPinningEvent, NodeEvent, RingUpdateEvent, TitanhEvent},
	utils::ref_builder::{AtomicRef, MutableRef, Ref},
};
use anyhow::Result;
use api::{
	common_types::{BlockHash, BlockInfo, BlockNumber},
	pinning_committee_types::{NodeId, PinningRing},
	TitanhApi,
};

pub struct SubstratePinningClient {
	api: TitanhApi,
	/// The node id bounded to the client
	node_id: NodeId,
	/// A reference to the pinning ring
	ring: AtomicRef<PinningRing>,
}

impl SubstratePinningClient {
	pub fn new(api: TitanhApi, node_id: NodeId, ring: AtomicRef<PinningRing>) -> Self {
		SubstratePinningClient { api, node_id, ring }
	}

	/// Return the events of a given block hash, and returns the list of events that are relevant to the pinning node, based on the pinning ring.
	pub async fn events_at(&self, block: BlockInfo) -> Result<Vec<NodeEvent>> {
		// Events at block identified by `block_hash`
		let runtime_events = self.api.runtime_events(Some(block.hash)).await?;

		let mut events = Vec::new();
		for event_record in runtime_events.into_iter() {
			let event = events::try_event_from_runtime(event_record.event);

			if let Some(event) = event {
				match event {
					TitanhEvent::Capsules(capsule_evenet) => {
						// If the pinning node is responsible for the key, then we get the partition number to which the key belongs. Else, we ignore the event.
						let maybe_partition =
							self.ring.key_node_partition(capsule_evenet.key, self.node_id)?;

						if let Some(partition) = maybe_partition {
							// The key belongs to the pinning node
							let pin_event = NodeEvent::pinning(partition, capsule_evenet);
							events.push(pin_event);
						}
					},
					TitanhEvent::PinningCommittee(ring_event) => {
						if self.ring.at() >= block.number {
							// The ring has been initialized at a later block, so we ignore the event
							continue;
						}
						match ring_event {
							// A new node has been added to the pinning ring, so update the ring and check if the pinning node is impacted, i.e., if it should drop keys
							RingUpdateEvent::NewPinningNode(node_id) => {
								let registration_event = NodeEvent::node_registration(node_id);
								events.push(registration_event);
							},
							RingUpdateEvent::RemovePinningNode {
								node_id,
								block_num,
								keytable_cid,
							} => {
								let removal_event =
									NodeEvent::node_removal(node_id, (block_num, keytable_cid));
								events.push(removal_event);
							},
						}
					},
				};
			}
		}
		// Add a block barrier event for later checkpointing
		events.push(NodeEvent::block_barrier(block.number));

		Ok(events)
	}

	/// Returns the list of events occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
	pub async fn events_in_range(
		&self,
		start: BlockNumber,
		end: BlockNumber,
	) -> Result<Vec<NodeEvent>> {
		let mut events = Vec::new();

		for block_number in start..=end {
			let block =
				BlockInfo { number: block_number, hash: self.api.block_hash(block_number).await? };

			let node_events = self.events_at(block).await?;
			events.extend(node_events);
		}

		Ok(events)
	}

	pub async fn partitioned_capsule_events_at(
		&self,
		at: BlockHash,
		break_event_idx: usize,
		partition_num: usize,
	) -> Result<Vec<KeyedPinningEvent>> {
		let runtime_events = self.api.runtime_events(Some(at)).await?;

		let mut partitioned_capsule_events = Vec::new();
		for (index, event_record) in runtime_events.into_iter().enumerate() {
			if index == break_event_idx {
				break;
			}
			let event = events::try_event_from_runtime(event_record.event);
			if let Some(event) = event {
				if let TitanhEvent::Capsules(capsule_event) = event {
					let maybe_partition =
						self.ring.key_node_partition(capsule_event.key, self.node_id)?;

					if let Some(partition) = maybe_partition {
						if partition == partition_num {
							partitioned_capsule_events.push(capsule_event);
						}
					}
				}
			}
		}

		Ok(partitioned_capsule_events)
	}

	pub fn api(&self) -> &TitanhApi {
		&self.api
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
}
