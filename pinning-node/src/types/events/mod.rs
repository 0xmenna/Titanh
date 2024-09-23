use crate::db::checkpointing::BarrierCheckpoint;
use api::{
	capsules_types::CapsuleKey,
	titanh::{capsules::Event, runtime_types::titanh_runtime::RuntimeEvent},
};

use super::cid::Cid;

pub mod dispatcher;
pub mod events_pool;

#[derive(Clone)]
pub enum PinningEvent {
	Pin { cid: Cid },
	RemovePin { cid: Cid },
	UpdatePin { old_cid: Cid, new_cid: Cid },
}

#[derive(Clone)]
pub struct KeyedPinningEvent {
	pub key: CapsuleKey,
	pub event: PinningEvent,
}

#[derive(Clone)]
pub enum NodeEvent {
	/// Pinning associated event
	Pinning(KeyedPinningEvent),
	// Control event to checkpoint events that have been processed at a given block
	BlockCheckpoint(BarrierCheckpoint),
}

impl From<KeyedPinningEvent> for NodeEvent {
	fn from(e: KeyedPinningEvent) -> Self {
		NodeEvent::Pinning(e)
	}
}

// Generates a pinning event from a runtime event. If the runtime event is not an event of interest it returns ⁠ None⁠.
pub fn try_pinning_event_from_runtime(event: RuntimeEvent) -> Option<KeyedPinningEvent> {
	let mut pinning_capsule_event = None;

	if let RuntimeEvent::Capsules(event) = event {
		// Capsule event
		match event {
			// Upload event
			Event::CapsuleUploaded { id, cid, .. } => {
				// If the cid is not in a valid format it means the event is not valid, so we return `None`
				let cid = cid.try_into().ok()?;
				pinning_capsule_event =
					Some(KeyedPinningEvent { key: id, event: PinningEvent::Pin { cid } })
			},
			// Update event
			Event::CapsuleContentChanged { capsule_id, old_cid, cid, .. } => {
				// Invalid cids bring to an invalid event, so return `None`
				let old_cid = old_cid.try_into().ok()?;
				let new_cid = cid.try_into().ok()?;
				pinning_capsule_event = Some(KeyedPinningEvent {
					key: capsule_id,
					event: PinningEvent::UpdatePin { old_cid, new_cid },
				})
			},
			// Deletion event
			Event::CapsuleDeleted { capsule_id, cid } => {
				let cid = cid.try_into().ok()?;
				pinning_capsule_event = Some(KeyedPinningEvent {
					key: capsule_id,
					event: PinningEvent::RemovePin { cid },
				})
			},
			// ignore
			_ => {},
		}
	}
	pinning_capsule_event
}
