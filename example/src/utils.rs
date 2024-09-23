use crate::types::CertificateId;
use serde::Deserialize;
use std::fs::{self};

#[derive(Deserialize, Clone)]
pub struct CertificateConfig {
	pub id: CertificateId,
	pub student_name: String,
	pub degree_program: String,
	pub graduation_year: String,
	pub grade: String,
	pub path: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
	pub chain_rpc: String,
	pub seed: String,
	pub ipfs_rpc: String,
	pub certificate: CertificateConfig,
}

impl Config {
	// Read config from a JSON file
	pub fn from_json() -> Config {
		let file_content: String =
			fs::read_to_string("data/config.json").expect("Failed to read the config file");
		let config: Config =
			serde_json::from_str(&file_content).expect("Failed to parse the config file");

		return config;
	}
}
