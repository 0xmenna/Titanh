use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::{HashOf, PinningNodeIdOf};
use scale_info::TypeInfo;
use sp_application_crypto::RuntimeAppPublic;
use sp_core::RuntimeDebug;
use sp_runtime::BoundedVec;
use sp_std::{prelude::*, vec::Vec};

/// The number of pinning nodes that will pin some content
pub type ReplicationFactor = u32;
/// The pinning nodes in the ring
pub type PinningNodes<T> = Vec<PinningNodeIdOf<T>>;
/// Ipfs keys of a pinning node
pub type IpfsKeys<T> = Vec<<T as Config>::IPFSNodeId>;
// Identifier that points to the content to pin
pub type ContentIdOf<T> = HashOf<T>;
/// The ring of pinning nodes
pub type PinningRing<T> = BoundedVec<PinningNodeIdOf<T>, <T as Config>::MaxPinningNodes>;

/// The key table of a pinning node that is leaving the ring. Updated at the blocknumber `block_num`. It contains the cid that points to the encoded key table
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct KeyTableAt<Blocknumber> {
	pub block_num: Blocknumber,
	pub cid: Vec<u8>,
}

/// The registration message of a pinning node
#[derive(Encode, Decode, MaxEncodedLen, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct RegistrationMessage<IPFSKey, Singature> {
	pub key: IPFSKey,
	pub signature: Singature,
}

pub type RegistrationMessageOf<T> = RegistrationMessage<
	<T as Config>::IPFSNodeId,
	<<T as Config>::IPFSNodeId as RuntimeAppPublic>::Signature,
>;

pub type PinningNodeIndex = u32;

/// The effect of a pinning node registration
#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum Registration<IpfsKey> {
	Addition,
	Substitution(IpfsKey),
}
