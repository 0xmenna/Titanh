use api::pinning_committee_types::NodeId;
use serde::Deserialize;
use sp_core::{Blake2Hasher, Hasher};
use std::fs;

#[derive(Deserialize)]
pub struct PeersConfig {
    pub ipfs_peers: Vec<IpfsPeer>,
}

impl PeersConfig {
    pub fn from_json(path: &str) -> Self {
        let file_content: String =
            fs::read_to_string(path).expect("Failed to read the config file");
        let peers_config: PeersConfig =
            serde_json::from_str(&file_content).expect("Failed to parse the config file");

        peers_config
    }
}

#[derive(Deserialize, Debug)]
pub struct IpfsPeer {
    pub rpc_url: String,
    pub peer_pubkey: String,
}

#[derive(Debug)]
pub struct Config {
    /// The seed phrase of the validator
    pub seed_phrase: String,
    /// The endpoint of the chain rpc node
    pub chain_node_endpoint: String,
    /// The list of IPFS peers
    pub ipfs_peers: Vec<IpfsPeer>,
    /// The number of retries for a failed pinning operation
    pub failure_retry: u8,
}

impl Config {
    // Read config from a JSON file
    pub fn new(
        seed_phrase: String,
        chain_node_endpoint: String,
        failure_retry: u8,
        ipfs_peers: Vec<IpfsPeer>,
    ) -> Self {
        Self {
            seed_phrase,
            chain_node_endpoint,
            ipfs_peers,
            failure_retry,
        }
    }

    pub fn node_id(&self) -> NodeId {
        // node_id = hash(ipfs_peer1 || ipfs_peer2 || ...)
        let mut ids = Vec::new();

        for peer in &self.ipfs_peers {
            let pubkey = hex::decode(&peer.peer_pubkey).expect("Invalid peer pubkey");
            ids.extend_from_slice(&pubkey);
        }

        Blake2Hasher::hash(&ids)
    }

    pub fn rpc_replicas(&self) -> Vec<&str> {
        self.ipfs_peers
            .iter()
            .map(|peer| peer.rpc_url.as_str())
            .collect()
    }
}
