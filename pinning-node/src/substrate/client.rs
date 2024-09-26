use crate::{
	types::events::{self, KeyedPinningEvent, NodeEvent, RingUpdateEvent, TitanhEvent},
	utils::ref_builder::MutableRef,
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
	ring: MutableRef<PinningRing>,
}

impl SubstratePinningClient {
	pub fn new(api: TitanhApi, node_id: NodeId, ring: MutableRef<PinningRing>) -> Self {
		SubstratePinningClient { api, node_id, ring }
	}

	/// It manages the events of a given block hash, and returns the list of events that are relevant to the pinning node, based on the pinning ring.
	pub async fn manage_events_at(&mut self, block: BlockInfo) -> Result<Vec<NodeEvent>> {
		// Events at block identified by `block_hash`
		let runtime_events = self.api.runtime_events(Some(block.hash)).await?;

		let mut events = Vec::new();
		for (event_idx, event_record) in runtime_events.into_iter().enumerate() {
			let event = events::try_event_from_runtime(event_record.event);

			if let Some(event) = event {
				match event {
					TitanhEvent::Capsules(mut capsule_evenet) => {
						// If the pinning node is responsible for the key, then we get the partition number to which the key belongs. Else, we ignore the event.
						let maybe_partition = self
							.ring
							.borrow()
							.key_node_partition(capsule_evenet.key, self.node_id)?;

						if let Some(partition) = maybe_partition {
							capsule_evenet.partition(partition);
							// The key belongs to the pinning node
							let pin_event = NodeEvent::pinning(capsule_evenet);
							events.push(pin_event);
						}
					},
					TitanhEvent::PinningCommittee(ring_event) => {
						let mut ring = self.ring.borrow_mut();

						if ring.at() >= block.number {
							// The ring has been initialized at a later block, so we ignore the event
							continue;
						}
						match ring_event {
							// A new node has been added to the pinning ring, so update the ring and check if the pinning node is impacted, i.e., if it should drop keys
							RingUpdateEvent::NewPinningNode(node_id) => {
								let idx = ring.insert_node(&node_id)?;

								let dist = ring.distance_from_idx(idx, &self.node_id)?;
								if dist <= ring.replication() {
									events.push(NodeEvent::NodeRegistration(dist));
								}
							},
							RingUpdateEvent::RemovePinningNode {
								node_id,
								block_num,
								keytable_cid,
							} => {
								let dist = ring.distance_between(&self.node_id, &node_id)?;
								ring.remove_node(&node_id)?;

								if dist <= ring.replication() {
									let keytable_block = block_num;

									debug_assert!(dist > 0);

									let rm_node_event = NodeEvent::node_rm_event(
										dist,
										keytable_block,
										keytable_cid,
									);
									events.push(rm_node_event);

									// Retrieve capsule events from the keytable block number to the current block number, because the keytable is valid up to the block number
									for block_num in keytable_block + 1..=block.number {
										let block_hash = self.api.block_hash(block_num).await?;
										let partition_num = ring.replication() as usize;

										let mut capsule_events = self
											.partitioned_capsule_events_at(
												block_hash,
												event_idx,
												partition_num,
											)
											.await?;

										if dist == ring.replication() {
											capsule_events.sort_by_key(|event| event.key);

											let barrier_idx = capsule_events
												.binary_search_by_key(&node_id, |event| event.key)
												.unwrap_or_else(|idx| idx);

											let capsule_events = &capsule_events[..barrier_idx];
											capsule_events.iter().for_each(|event| {
												let pin_event = NodeEvent::pinning_event(
													partition_num,
													event.clone(),
												);
												events.push(pin_event);
											});
										}
									}
								}
							},
						}
					},
				};
			}
		}
		Ok(events)
	}

	/// Returns the list of events of type `EventType` occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
	pub async fn manage_events_in_range(
		&mut self,
		start: BlockNumber,
		end: BlockNumber,
	) -> Result<Vec<NodeEvent>> {
		let mut events = Vec::new();

		for block_number in start..=end {
			let block =
				BlockInfo { number: block_number, hash: self.api.block_hash(block_number).await? };

			let node_events = self.manage_events_at(block).await?;
			events.extend(node_events);
		}

		Ok(events)
	}

	async fn partitioned_capsule_events_at(
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
						self.ring.borrow().key_node_partition(capsule_event.key, self.node_id)?;

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
