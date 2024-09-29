use anyhow::Result;
use api::{common_types::KeyPair, pinning_committee_types::NodeId};
use codec::Encode;
use serde::Deserialize;
use sp_core::{Blake2Hasher, Hasher, Pair};
use std::fs;

#[derive(Deserialize)]
pub struct IpfsPeer {
    pub rpc_url: String,
    pub peer_pubkey: String,
}

#[derive(Deserialize)]
pub struct Config {
    /// The node id within the same validator node
    pub node_id: u32,
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
    pub fn from_json() -> Config {
        let file_content: String = fs::read_to_string("config/pinning-config.json")
            .expect("Failed to read the config file");
        let config: Config =
            serde_json::from_str(&file_content).expect("Failed to parse the config file");

        return config;
    }

    pub fn node_id(&self) -> NodeId {
        // node_id = hash(validator_id || node_id || ipfs_peer1 || ipfs_peer2 || ...)
        let mut ids = Vec::new();

        let validator_id = KeyPair::from_string(&self.seed_phrase, None)
            .expect("Invalid seed phrase")
            .public();
        ids.extend_from_slice(&validator_id.encode());

        ids.extend_from_slice(&self.node_id.encode());

        for peer in &self.ipfs_peers {
            let pubkey = decode_hex_str(&peer.peer_pubkey).expect("Invalid hex pubkey");
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

pub fn decode_hex_str(hex_str: &str) -> Result<Vec<u8>> {
    if !hex_str[..2].starts_with("0x") {
        return Err(anyhow::anyhow!("Hex string should start with 0x"));
    }
    let hex_str = &hex_str[2..];
    let decoded_hex = hex::decode(hex_str)?;

    Ok(decoded_hex)
}
