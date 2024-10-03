use super::titanh;
use anyhow::Result;
use codec::{Decode, Encode};
use sp_core::H256;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::blocks::ExtrinsicEvents;
use subxt::utils::AccountId32;
use subxt::{blocks::BlockRef, tx::PairSigner, OnlineClient, SubstrateConfig};

#[derive(Copy, Clone, Encode, Decode)]
pub struct BlockInfo {
    pub number: BlockNumber,
    pub hash: BlockHash,
}

impl BlockInfo {
    pub fn new(number: BlockNumber, hash: BlockHash) -> Self {
        Self { number, hash }
    }
}

pub type BlockNumber = titanh::system::storage::types::number::Number;
/// The key pair used by the validator
pub type KeyPair = sp_core::sr25519::Pair;
/// The substrate api
pub type SubstrateApi = OnlineClient<SubstrateConfig>;
/// Signer used in the api transactions
pub type Signer = PairSigner<SubstrateConfig, KeyPair>;
/// Chain's Rpc methods
pub type Rpc = LegacyRpcMethods<SubstrateConfig>;
/// The events of the chain to be used in the api
pub type Events = ExtrinsicEvents<SubstrateConfig>;

#[derive(Clone, Encode, Decode, Copy)]
pub struct BlockHash(pub H256);

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

pub struct User(AccountId32);

impl User {
    pub fn from_pubkey(pubkey: &str) -> Result<Self> {
        let mut account = [0u8; 32];
        if pubkey.starts_with("0x") {
            let pubkey = &pubkey[2..];
            let pubkey = hex::decode(pubkey)?;
            if pubkey.len() != 32 {
                return Err(anyhow::anyhow!("Invalid user public key"));
            }
            account.copy_from_slice(&pubkey);

            Ok(User(AccountId32::from(account)))
        } else {
            Err(anyhow::anyhow!("User public key is not in hex format"))
        }
    }

    pub fn account(&self) -> AccountId32 {
        self.0.clone()
    }
}

#[derive(Default)]
pub enum ConsistencyLevel {
    // This level of consistency reflects an eventual consistency model => Transaction is valid and included in the transaction pool. Not yet processed in a block.
    Low,
    // This level is still eventual but with a higher probability of consistency. Transaction is included in a block but not yet finalized. There could be chain forks that could revert the transaction.
    #[default]
    Medium,
    // This level is the highest level of consistency. Transaction is included in a block and the block is finalized. All transactions in the block are ordered and irreversible. At the finalized state, everyone sees the same order of transactions and the latest write to the chain state.
    High,
}
