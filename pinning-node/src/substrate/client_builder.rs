use super::client::SubstrateClient;
use crate::types::chain::{Rpc, SubstrateApi, ValidatorKeyPair};
use anyhow::Result;
use sp_core::Pair;
use subxt::{backend::rpc::RpcClient, tx::PairSigner, SubstrateConfig};
use url::Url;

/// Builder for the Substrate Client
pub struct SubstrateClientBuilder<T>(T);

impl SubstrateClientBuilder<ApiNotInitialized> {
	/// Creates a client with a local rpc configuration
	pub fn new() -> SubstrateClientBuilder<RpcEndpoint> {
		Self::from_url("ws://127.0.0.1:9944")
	}

	/// Creates a client with a custom rpc configuration
	pub fn from_url(url: &str) -> SubstrateClientBuilder<RpcEndpoint> {
		let url = Url::parse(url).expect("Invalid URL");

		SubstrateClientBuilder(RpcEndpoint(url))
	}
}

impl SubstrateClientBuilder<RpcEndpoint> {
	/// Creates a client from a valid seed phrase
	pub fn keyring_material(
		self,
		phrase: &str,
		password: Option<&str>,
	) -> SubstrateClientBuilder<ApiReady> {
		// Derive the key pair from the seed phrase (mnemonic)
		let key_pair = ValidatorKeyPair::from_string(phrase, password).expect("Invalid key pair");

		// Create a signer using the key pair
		let signer = PairSigner::new(key_pair);

		SubstrateClientBuilder(ApiReady { rpc_url: self.0, signer })
	}
}

impl SubstrateClientBuilder<ApiReady> {
	pub async fn build(self) -> SubstrateClient {
		// First, create a raw RPC client
		let rpc_client = RpcClient::from_url(self.0.rpc_url.as_str()).await.expect("No RPC client");

		// Use this to construct the RPC methods
		let rpc = Rpc::new(rpc_client.clone());

		// We can use the same client to drive our full Subxt interface
		let api = SubstrateApi::from_rpc_client(rpc_client.clone())
			.await
			.expect("Invalid Substrate API");

		SubstrateClient::new(api, rpc, self.0.signer)
	}
}

#[derive(Default)]
pub struct ApiNotInitialized;

pub struct RpcEndpoint(Url);

impl RpcEndpoint {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

pub struct ApiReady {
	// An rpc endpoint for the chain's operations
	rpc_url: RpcEndpoint,
	// A valid signer for transactions
	signer: PairSigner<SubstrateConfig, ValidatorKeyPair>,
}
