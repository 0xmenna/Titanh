use primitives::Hash;
use sp_core::H256;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::{blocks::BlockRef, tx::PairSigner, OnlineClient, SubstrateConfig};

/// Module for accessing all blockchain related types. It is based on the encoded metadata provided at `runtime_metadata_path`
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod titanh {}

/// The key pair used by the validator
pub type ValidatorKeyPair = sp_core::sr25519::Pair;
/// The substrate api
pub type SubstrateApi = OnlineClient<SubstrateConfig>;
/// Signer used in the api transactions
pub type Signer = PairSigner<SubstrateConfig, ValidatorKeyPair>;
/// Chain's Rpc methods
pub type Rpc = LegacyRpcMethods<SubstrateConfig>;
/// A pinning node's identifier in the ring
pub type NodeId = Hash;

pub struct BlockHash(Hash);

impl From<BlockHash> for BlockRef<H256> {
	fn from(block_hash: BlockHash) -> Self {
		BlockRef::from(block_hash)
	}
}

impl From<Hash> for BlockHash {
	fn from(h: H256) -> Self {
		BlockHash(h)
	}
}

pub type CapsuleKey = Hash;
