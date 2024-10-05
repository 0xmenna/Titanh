use crate::{
    common_types::{BlockInfo, BlockNumber, ConsistencyLevel},
    titanh::{
        self,
        runtime_types::{
            pallet_pinning_committee::types::KeyTableAt, primitives::ed25519::app_ed25519,
        },
        utility::calls::types::batch_all::Calls as RuntimeCalls,
    },
    TitanhApi,
};
use anyhow::Result;
use codec::Decode;
use crypto::IpfsPair;
use sp_core::{Blake2Hasher, Hasher, H256};
use types::PinningRing;

pub struct PinningCommitteeApi<'a> {
    titanh: &'a TitanhApi,
    ipfs_peers: Option<Vec<IpfsPair>>,
}

impl<'a> From<&'a TitanhApi> for PinningCommitteeApi<'a> {
    fn from(titanh: &'a TitanhApi) -> Self {
        PinningCommitteeApi {
            titanh,
            ipfs_peers: None,
        }
    }
}

impl PinningCommitteeApi<'_> {
    /// Provides the seeds of the IPFS peers associated to a validator's pinning
    pub fn ipfs_seeds(self, ipfs_peers_seed: Vec<Vec<u8>>) -> Result<Self> {
        let mut ipfs_peers = Vec::new();
        for seed in ipfs_peers_seed {
            let pair = IpfsPair::from_seed(&seed).map_err(|_| anyhow::anyhow!("Invalid seed"))?;
            ipfs_peers.push(pair);
        }

        Ok(Self {
            ipfs_peers: Some(ipfs_peers),
            ..self
        })
    }

    pub async fn pinning_ring_at(&mut self, block: BlockInfo) -> Result<PinningRing> {
        let ring_query = titanh::storage().pinning_committee().pinning_nodes_ring();
        let ring = self.titanh.query(&ring_query, Some(block.hash)).await?;
        let ring = ring.0.to_vec();

        let replication_factor_query = titanh::storage()
            .pinning_committee()
            .content_replication_factor();
        let replication_factor = self
            .titanh
            .query(&replication_factor_query, Some(block.hash))
            .await?;

        let pinning_ring = PinningRing::new(ring, replication_factor, block);
        Ok(pinning_ring)
    }

    pub async fn set_committe_config(
        &self,
        rep_factor: u32,
        ipfs_replicas: u32,
        pinning_nodes: u32,
    ) -> Result<H256> {
        let mut calls = RuntimeCalls::new();

        let rep_factor_call = calls::build_rep_factor_call(rep_factor);
        calls.push(rep_factor_call);

        let replicas_call = calls::build_ipfs_replicas_call(ipfs_replicas);
        calls.push(replicas_call);

        let pinning_nodes_call = calls::build_pinning_nodes_call(pinning_nodes);
        calls.push(pinning_nodes_call);

        let tx_hash = self
            .titanh
            .sign_and_submit_batch(calls, ConsistencyLevel::High)
            .await?;

        Ok(tx_hash)
    }

    pub async fn register_ipfs_peers(&self) -> Result<H256> {
        let mut calls = RuntimeCalls::new();

        if let Some(ipfs_peers) = &self.ipfs_peers {
            for ipfs_pair in ipfs_peers {
                let registration_call = calls::build_registration_message_call(ipfs_pair);
                calls.push(registration_call);
            }

            let tx_hash = self
                .titanh
                .sign_and_submit_batch(calls, ConsistencyLevel::High)
                .await?;

            Ok(tx_hash)
        } else {
            Err(anyhow::anyhow!("IPFS peers keys are not set"))
        }
    }

    /// Leaves the committee, providing the block number until which the node has checkpointed its data. Its data is pointed by the list of cids provided as `Vec<Vec<u8>>`.
    pub async fn leave_committee(
        &self,
        cids: Vec<Vec<u8>>,
        block_num: BlockNumber,
    ) -> Result<H256> {
        let pinning_node = self.compute_pinning_node_id()?;
        let leave_tx = titanh::tx()
            .pinning_committee()
            .rm_pinning_node(pinning_node, KeyTableAt { block_num, cids });

        let tx_hash = self
            .titanh
            .sign_and_submit_wait_finalized(&leave_tx)
            .await?;

        Ok(tx_hash.extrinsic_hash())
    }

    pub fn compute_pinning_node_id(&self) -> Result<H256> {
        if let Some(ipfs_peers) = &self.ipfs_peers {
            let mut ids = Vec::new();

            for ipfs_peer in ipfs_peers {
                ids.extend_from_slice(&ipfs_peer.public());
            }

            Ok(Blake2Hasher::hash(&ids[..]))
        } else {
            Err(anyhow::anyhow!("IPFS peers keys are not set"))
        }
    }
}

impl TryFrom<Vec<u8>> for app_ed25519::Public {
    type Error = ();

    fn try_from(public: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        app_ed25519::Public::decode(&mut &public[..]).map_err(|_| ())
    }
}

impl TryFrom<Vec<u8>> for app_ed25519::Signature {
    type Error = ();

    fn try_from(signature: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        app_ed25519::Signature::decode(&mut &signature[..]).map_err(|_| ())
    }
}

pub mod calls;
pub mod crypto;
pub mod types;
