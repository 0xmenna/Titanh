use super::client::SubstrateClient;
use crate::{
    db::checkpointing::DbCheckpoint,
    utils::{config::Config, traits::ClientBuilder},
};
use api::{common_types::BlockInfo, pinning_committee_types::NodeId, TitanhApiBuilder};
use async_trait::async_trait;

pub struct SubstratePinningConfig<'a> {
    /// The node id of the pinning node
    pub node_id: NodeId,
    /// The virtual node instance within all the nodes running in the same machine
    pub virtual_node_idx: u32,
    /// The rpc url of the substrate node
    pub rpc_url: &'a str,
    /// The seed phrase of the validator
    pub seed_phrase: &'a str,
}

impl<'a> From<&'a Config> for SubstratePinningConfig<'a> {
    fn from(config: &'a Config) -> Self {
        SubstratePinningConfig {
            node_id: config.node_id(),
            virtual_node_idx: config.idx,
            rpc_url: &config.chain_node_endpoint,
            seed_phrase: &config.seed_phrase,
        }
    }
}

pub struct SubstrateClientBuilder<'a> {
    config: SubstratePinningConfig<'a>,
}

#[async_trait]
impl<'a> ClientBuilder<'a, SubstrateClient> for SubstrateClientBuilder<'a> {
    fn from_config(config: &'a Config) -> Self {
        let config = SubstratePinningConfig::from(config);
        Self { config }
    }

    async fn build(self) -> SubstrateClient {
        let api = TitanhApiBuilder::rpc(&self.config.rpc_url)
            .seed(&self.config.seed_phrase)
            .build()
            .await;

        let maybe_block_num = DbCheckpoint::get_blocknumber_from_db_node(
            self.config.virtual_node_idx,
            self.config.node_id,
        );
        let block = if let Some(block_num) = maybe_block_num {
            let hash = api.block_hash(block_num).await.unwrap();
            BlockInfo::new(block_num, hash)
        } else {
            api.latest_finalized_block().await.unwrap()
        };

        SubstrateClient::new(api, self.config.node_id, block)
    }
}
