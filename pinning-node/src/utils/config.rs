use anyhow::Result;
use api::pinning_committee_types::NodeId;
use serde::Deserialize;
use sp_core::{Blake2Hasher, Hasher};
use std::fs;

#[derive(Deserialize)]
pub struct IpfsPeer {
	pub rpc_url: String,
	pub peer_pubkey: String,
}

#[derive(Deserialize)]
pub struct Config {
	pub seed_phrase: String,
	pub chain_node_endpoint: String,
	pub ipfs_peers: Vec<IpfsPeer>,
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
		let ipfs_peers = &self.ipfs_peers;
		let mut ipfs_keys = Vec::new();
		// Print the IPFS peers
		for peer in ipfs_peers {
			let pubkey = decode_hex_str(&peer.peer_pubkey).unwrap();
			ipfs_keys.extend_from_slice(&pubkey);
		}

		Blake2Hasher::hash(&ipfs_keys)
	}

	pub fn rpc_replicas(&self) -> Vec<&str> {
		self.ipfs_peers.iter().map(|peer| peer.rpc_url.as_str()).collect()
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
