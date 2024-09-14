use sp_core::H256;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::blocks::BlockRef;
use subxt::tx::PairSigner;
use subxt::OnlineClient;
use subxt::SubstrateConfig;
/// The chain's runtime
pub use titanh_runtime::Runtime as TitanhRuntimeConfig;
// An event that takes place after a chain state transition
pub use titanh_runtime::RuntimeEvent;

use super::ipfs::Cid;

pub type ValidatorKeyPair = sp_core::sr25519::Pair;

/// The substrate api
pub type SubstrateApi = OnlineClient<SubstrateConfig>;

/// Signer used in the api transactions
pub type Signer = PairSigner<SubstrateConfig, ValidatorKeyPair>;

/// Chain's Rpc methods
pub type Rpc = LegacyRpcMethods<SubstrateConfig>;

pub struct BlockHash(H256);

impl From<BlockHash> for BlockRef<H256> {
	fn from(block_hash: BlockHash) -> Self {
		BlockRef::from(block_hash)
	}
}

impl From<H256> for BlockHash {
	fn from(h: H256) -> Self {
		BlockHash(h)
	}
}

pub type CapsuleKey = Vec<u8>;

pub enum CapsuleEvents {
	Upload { cid: Cid, key: CapsuleKey },
	Update { key: CapsuleKey, old_cid: Cid, new_cid: Cid },
	Removal { key: CapsuleKey, cid: Cid },
}
