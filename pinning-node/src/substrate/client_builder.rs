use super::client::{SubstrateClient, SubstratePinningClient};
use crate::{
	types::chain::{NodeId, Rpc, SubstrateApi, ValidatorKeyPair},
	utils::{config::Config, ref_builder, traits::ClientBuilder},
};
use async_trait::async_trait;
use sp_core::Pair;
use subxt::{backend::rpc::RpcClient, tx::PairSigner};

pub struct SubstratePinningConfig<'a> {
	/// The node id of the pinning node
	pub node_id: NodeId,
	/// The rpc url of the substrate node
	pub rpc_url: &'a str,
	/// The seed phrase of the validator
	pub seed_phrase: &'a str,
	/// The password of the seed phrase
	pub password: Option<&'a str>,
}

impl<'a> From<&'a Config> for SubstratePinningConfig<'a> {
	fn from(config: &'a Config) -> Self {
		SubstratePinningConfig {
			node_id: config.node_id(),
			rpc_url: &config.chain_node_endpoint,
			seed_phrase: &config.seed_phrase,
			password: None,
		}
	}
}

pub struct SubstrateClientBuilder<'a> {
	config: SubstratePinningConfig<'a>,
}

#[async_trait]
impl<'a> ClientBuilder<'a, SubstratePinningClient> for SubstrateClientBuilder<'a> {
	fn from_config(config: &'a Config) -> Self {
		let config = SubstratePinningConfig::from(config);
		Self { config }
	}

	async fn build(self) -> SubstratePinningClient {
		// Derive the key pair from the seed phrase (mnemonic)
		let key_pair = ValidatorKeyPair::from_string(self.config.seed_phrase, self.config.password)
			.expect("Invalid key pair");

		// Create a signer using the key pair
		let signer = PairSigner::new(key_pair);
		let rpc_client = RpcClient::from_url(self.config.rpc_url).await.expect("No RPC client");

		// Use this to construct the RPC methods
		let rpc = Rpc::new(rpc_client.clone());

		// We can use the same client to drive our full Subxt interface
		let api = SubstrateApi::from_rpc_client(rpc_client.clone())
			.await
			.expect("Invalid Substrate API");

		let client = SubstrateClient::new(api, rpc, signer);

		let ring = client
			.ring_state()
			.await
			.expect("Ring is expected to be initialized during substrate client initialization");
		let ring = ref_builder::create_atomic_ref(ring);

		SubstratePinningClient::new(client, self.config.node_id, ring)
	}
}
