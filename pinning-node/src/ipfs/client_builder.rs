use super::client::IpfsClient;
use crate::{types::cid::Cid, utils::config::Config};
use anyhow::Result;
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
    cid_pins: Vec<(Cid, u32)>,
}

const MAX_REPLICAS: usize = 10;

impl<'a> IpfsClientBuilder<'a> {
    pub fn from_config(config: &'a Config, cid_pins: Vec<(Cid, u32)>) -> Self {
        let config = IpfsConfig::from(config);
        Self { config, cid_pins }
    }

    pub async fn build(self) -> Result<IpfsClient> {
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
        Ok(IpfsClient::new(
            replicas,
            self.config.failure_retry,
            self.cid_pins,
        ))
    }
}
