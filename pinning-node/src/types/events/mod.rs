use super::cid::Cid;
use api::{
	capsules_types::CapsuleKey,
	common_types::BlockNumber,
	pinning_committee_types::NodeId,
	titanh::{
		capsules::Event as CapsuleEvent, pinning_committee::Event as PinningCommitteeEvent,
		runtime_types::titanh_runtime::RuntimeEvent,
	},
};

pub mod dispatcher;
pub mod events_pool;

#[derive(Clone)]
pub enum PinningEvent {
	Pin { cid: Cid },
	RemovePin { cid: Cid },
	UpdatePin { old_cid: Cid, new_cid: Cid },
}

#[derive(Clone)]
pub enum RingUpdateEvent {
	NewPinningNode(NodeId),
	RemovePinningNode { node_id: NodeId, block_num: BlockNumber, keytable_cid: Cid },
}

#[derive(Clone)]
pub struct KeyedPinningEvent {
	pub key: CapsuleKey,
	pub event: PinningEvent,
}

pub enum TitanhEvent {
	Capsules(KeyedPinningEvent),
	PinningCommittee(RingUpdateEvent),
}

#[derive(Clone)]
pub enum NodeEvent {
	/// Pinning associated event
	Pinning {
		partition_num: usize,
		keyed_event: KeyedPinningEvent,
	},
	/// Control event to checkpoint events that have been processed at a given block
	BlockBarrier(BlockNumber),
	// An event of a new node registration
	NodeRegistration(NodeId),
	// Event of node removal
	NodeRemoval {
		node: NodeId,
		keytable: (BlockNumber, Cid),
	},
}

impl NodeEvent {
	pub fn pinning(partition_num: usize, keyed_event: KeyedPinningEvent) -> Self {
		NodeEvent::Pinning { partition_num, keyed_event }
	}

	pub fn node_registration(node: NodeId) -> Self {
		NodeEvent::NodeRegistration(node)
	}

	pub fn node_removal(node: NodeId, keytable: (BlockNumber, Cid)) -> Self {
		NodeEvent::NodeRemoval { node, keytable }
	}

	pub fn block_barrier(block_num: BlockNumber) -> Self {
		NodeEvent::BlockBarrier(block_num)
	}
}

// Generates a pinning event from a runtime event. If the runtime event is not an event of interest it returns ⁠ None⁠.
pub fn try_event_from_runtime(event: RuntimeEvent) -> Option<TitanhEvent> {
	let mut node_event = None;

	if let RuntimeEvent::Capsules(event) = event {
		// Capsule event
		match event {
			// Upload event
			CapsuleEvent::CapsuleUploaded { id, cid, .. } => {
				// If the cid is not in a valid format it means the event is not valid, so we return `None`
				let cid = cid.try_into().ok()?;
				node_event = Some(TitanhEvent::Capsules(KeyedPinningEvent {
					key: id,
					event: PinningEvent::Pin { cid },
				}))
			},
			// Update event
			CapsuleEvent::CapsuleContentChanged { capsule_id, old_cid, cid, .. } => {
				// Invalid cids bring to an invalid event, so return `None`
				let old_cid = old_cid.try_into().ok()?;
				let new_cid = cid.try_into().ok()?;
				node_event = Some(TitanhEvent::Capsules(KeyedPinningEvent {
					key: capsule_id,
					event: PinningEvent::UpdatePin { old_cid, new_cid },
				}))
			},
			// Deletion event
			CapsuleEvent::CapsuleDeleted { capsule_id, cid } => {
				let cid = cid.try_into().ok()?;
				node_event = Some(TitanhEvent::Capsules(KeyedPinningEvent {
					key: capsule_id,
					event: PinningEvent::RemovePin { cid },
				}))
			},
			// ignore
			_ => {},
		}
	} else if let RuntimeEvent::PinningCommittee(event) = event {
		// Pinning committee event
		match event {
			PinningCommitteeEvent::PinningNodeRegistration { pinning_node, .. } => {
				node_event = Some(TitanhEvent::PinningCommittee(RingUpdateEvent::NewPinningNode(
					pinning_node,
				)))
			},
			PinningCommitteeEvent::PinningNodeRemoval { pinning_node, key_table, .. } => {
				node_event =
					Some(TitanhEvent::PinningCommittee(RingUpdateEvent::RemovePinningNode {
						node_id: pinning_node,
						block_num: key_table.block_num,
						keytable_cid: key_table.cid.try_into().ok()?,
					}))
			},
			// ignore
			_ => {},
		}
	}

	node_event
}
