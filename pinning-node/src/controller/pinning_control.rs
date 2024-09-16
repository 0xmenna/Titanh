use std::rc::Rc;

use crate::{
	substrate::client::SubstrateClient,
	types::{
		chain::NodeId,
		checkpoint::PinningCheckpoint,
		pinning::{PinningEventsPool, PinningRing},
	},
};

pub struct PinningNodeController {
	/// The substrate client for chain queries
	substrate_client: SubstrateClient,
	/// The checkpoint for the pinning node
	checkpoint: PinningCheckpoint,
	/// The nodes ring, within the replication factor
	ring: Rc<PinningRing>,
	/// The node identifier in the ring
	id: NodeId,
	// TODO: remember to add ipfs client and channels
	events_pool: PinningEventsPool,
}
