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

pub mod batch;
pub mod dispatcher;
pub mod events_pool;

#[derive(Clone)]
pub enum NodeEvent {
	/// Pinning associated event
	Pinning(KeyedPinningEvent),
	/// Control event to checkpoint events that have been processed at a given block
	BlockBarrier(BlockBarrierEvent),
	// An event of a new node registration
	NodeRegistration(JoinNodeEvent),
	// Event of node removal
	NodeRemoval(LeaveNodeEvent),
}

impl NodeEvent {
	pub fn is_committee_event(&self) -> bool {
		match self {
			NodeEvent::NodeRegistration(_) | NodeEvent::NodeRemoval { .. } => true,
			_ => false,
		}
	}

	pub fn pinning_event(self) -> Option<KeyedPinningEvent> {
		match self {
			NodeEvent::Pinning(event) => Some(event),
			_ => None,
		}
	}

	pub fn block_barrier_event(self) -> Option<BlockNumber> {
		match self {
			NodeEvent::BlockBarrier(block_num) => Some(block_num),
			_ => None,
		}
	}
}

#[derive(Clone)]
pub struct KeyedPinningEvent {
	pub key: CapsuleKey,
	pub pin: PinningEvent,
}

#[derive(Clone)]
pub enum PinningEvent {
	Pin { cid: Cid },
	RemovePin { cid: Cid },
	UpdatePin { old_cid: Cid, new_cid: Cid },
}

pub type JoinNodeEvent = NodeId;

pub type BlockBarrierEvent = BlockNumber;

#[derive(Clone)]
pub struct LeaveNodeEvent {
	node: NodeId,
	transferred_keytable: (BlockNumber, Vec<Cid>),
}

impl LeaveNodeEvent {
	pub fn node(&self) -> NodeId {
		self.node
	}

	pub fn key_table_at(&self) -> BlockNumber {
		self.transferred_keytable.0
	}

	pub fn row_cid_of(&self, row_idx: usize) -> Cid {
		self.transferred_keytable.1[row_idx]
	}
}

// Generates a pinning event from a runtime event. If the runtime event is not an event of interest it returns ⁠ None⁠.
pub fn try_event_from_runtime(event: RuntimeEvent) -> Option<NodeEvent> {
	let mut node_event = None;

	if let RuntimeEvent::Capsules(event) = event {
		// Capsule event
		match event {
			// Upload event
			CapsuleEvent::CapsuleUploaded { id, cid, .. } => {
				// If the cid is not in a valid format it means the event is not valid, so we return `None`
				let cid = cid.try_into().ok()?;
				node_event = Some(NodeEvent::Pinning(KeyedPinningEvent {
					key: id,
					pin: PinningEvent::Pin { cid },
				}))
			},
			// Update event
			CapsuleEvent::CapsuleContentChanged { capsule_id, old_cid, cid, .. } => {
				// Invalid cids bring to an invalid event, so return `None`
				let old_cid = old_cid.try_into().ok()?;
				let new_cid = cid.try_into().ok()?;
				node_event = Some(NodeEvent::Pinning(KeyedPinningEvent {
					key: capsule_id,
					pin: PinningEvent::UpdatePin { old_cid, new_cid },
				}))
			},
			// Deletion event
			CapsuleEvent::CapsuleDeleted { capsule_id, cid } => {
				let cid = cid.try_into().ok()?;
				node_event = Some(NodeEvent::Pinning(KeyedPinningEvent {
					key: capsule_id,
					pin: PinningEvent::RemovePin { cid },
				}))
			},
			// ignore
			_ => {},
		}
	} else if let RuntimeEvent::PinningCommittee(event) = event {
		// Pinning committee event
		match event {
			PinningCommitteeEvent::PinningNodeRegistration { pinning_node, .. } => {
				node_event = Some(NodeEvent::NodeRegistration(pinning_node))
			},
			PinningCommitteeEvent::PinningNodeRemoval { pinning_node, key_table, .. } => {
				node_event = Some(NodeEvent::NodeRemoval(LeaveNodeEvent {
					node: pinning_node,
					transferred_rows: (
						key_table.block_num,
						key_table.cids.into_iter().map(|cid| cid.try_into().ok()?).collect(),
					),
				}))
			},
			// ignore
			_ => {},
		}
	}

	node_event
}
