use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::HashOf;
use scale_info::TypeInfo;
use sp_runtime::BoundedVec;
use sp_std::prelude::*;

/// The number of pinning nodes that will pin some content
pub type ReplicationFactor = u32;

/// Identifeier of a pinning node in the ring
pub type PinningNodeIdOf<T> = HashOf<T>;

/// Metdata of a pinning node
#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct PinningNodeMetadata<ValidatorId> {
	/// The IPFS HTTP gateway to contact for content retrieval
	pub gateway: u8,
	/// Public key of the validator node
	pub validator_id: ValidatorId,
	/// The IPFS id of the node
	pub ipfs_peer_id: u8,
	// Index of the pinning node in the ring
	pub idx: u32,
}

/// The ring of pinning nodes
pub type PinningNodesOf<T> = BoundedVec<PinningNodeIdOf<T>, <T as Config>::MaxPinningNodes>;

// Identifier that of the content to pin (basically the capsule id)
pub type ContentIdOf<T> = HashOf<T>;
