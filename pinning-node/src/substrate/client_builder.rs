use super::client2::SubstratePinningClient;
use crate::{
	db::checkpointing::DbCheckpoint,
	utils::{config::Config, ref_builder, traits::ClientBuilder},
};
use api::{pinning_committee_types::NodeId, TitanhApiBuilder};
use async_trait::async_trait;

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
		let api = TitanhApiBuilder::rpc(&self.config.rpc_url)
			.seed(&self.config.seed_phrase)
			.build()
			.await;

		let db = DbCheckpoint::new();
		let block_num = db
			.read_blocknumber_checkpoint()
			.expect("Failed to interact with the checkpointing db");

		let ring =
			api.pinning_committee().pinning_ring(block_num).await.expect(
				"Ring is expected to be initialized during substrate client initialization",
			);
		let ring = ref_builder::create_atomic_ref(ring);

		SubstratePinningClient::new(api, self.config.node_id, ring)
	}
}
