use super::titanh;
use sp_core::H256;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::{blocks::BlockRef, tx::PairSigner, OnlineClient, SubstrateConfig};

pub type BlockNumber = titanh::system::storage::types::number::Number;
/// The key pair used by the validator
pub type KeyPair = sp_core::sr25519::Pair;
/// The substrate api
pub type SubstrateApi = OnlineClient<SubstrateConfig>;
/// Signer used in the api transactions
pub type Signer = PairSigner<SubstrateConfig, KeyPair>;
/// Chain's Rpc methods
pub type Rpc = LegacyRpcMethods<SubstrateConfig>;
/// A pinning node's identifier in the ring
pub type NodeId = H256;

pub struct BlockHash(H256);

impl From<BlockHash> for BlockRef<H256> {
	fn from(block_hash: BlockHash) -> Self {
		BlockRef::from(block_hash.0)
	}
}

impl From<H256> for BlockHash {
	fn from(h: H256) -> Self {
		BlockHash(h)
	}
}

pub type CapsuleKey = H256;
