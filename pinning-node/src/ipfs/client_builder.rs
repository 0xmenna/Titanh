use super::client::IpfsClient;
use crate::utils::{config::Config, traits::ClientBuilder};
use anyhow::Result;
use async_trait::async_trait;
use ipfs_api_backend_hyper::{IpfsClient as ApiIpfsClient, TryFromUri};

pub struct IpfsConfig<'a> {
    pub rpc_replicas: Vec<&'a str>,
    pub failure_retry: u8,
}

impl<'a> From<&'a Config> for IpfsConfig<'a> {
    fn from(config: &'a Config) -> Self {
        IpfsConfig {
            rpc_replicas: config.rpc_replicas(),
            failure_retry: config.failure_retry,
        }
    }
}

pub struct IpfsClientBuilder<'a> {
    config: IpfsConfig<'a>,
}

const MAX_REPLICAS: usize = 10;

#[async_trait]
impl<'a> ClientBuilder<'a, Result<IpfsClient>> for IpfsClientBuilder<'a> {
    fn from_config(config: &'a Config) -> Self {
        let config = IpfsConfig::from(config);
        Self { config }
    }

    async fn build(self) -> Result<IpfsClient> {
        let replicas: Result<Vec<ApiIpfsClient>, _> = self
            .config
            .rpc_replicas
            .into_iter()
            .map(|url| ApiIpfsClient::from_str(url))
            .collect();

        let replicas = replicas?;
        if replicas.is_empty() {
            return Err(anyhow::anyhow!("No replicas provided"));
        }

        if replicas.len() > MAX_REPLICAS {
            return Err(anyhow::anyhow!(
                "Too many replicas provided, max is {}",
                MAX_REPLICAS
            ));
        }
        Ok(IpfsClient::new(replicas, self.config.failure_retry))
    }
}
