use crate::{
	substrate::client::SubstrateClient,
	types::{
		chain::NodeId,
		checkpoint::PinningCheckpoint,
		pinning::{PinningEventsPool, PinningRing},
	},
};
use ipfs_api_backend_hyper::IpfsClient;
use std::rc::Rc;

pub struct PinningNodeController {
	/// The IPFS client
	ipfs: IpfsClient,
	/// The substrate client for chain queries
	substrate_client: SubstrateClient,
	/// The checkpoint for the pinning node
	checkpoint: PinningCheckpoint,
	/// The nodes ring, within the replication factor
	ring: Rc<PinningRing>,
	/// The node identifier in the ring
	id: NodeId,
	/// Pool of events that handles event subscription
	events_pool: PinningEventsPool,
}

impl PinningNodeController {
	pub fn bootstrap() -> Self {
		
	}
}
