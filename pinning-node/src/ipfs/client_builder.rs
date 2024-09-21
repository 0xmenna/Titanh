use super::client::IpfsClient;
use crate::utils::{config::Config, traits::ClientBuilder};
use async_trait::async_trait;
use ipfs_api_backend_hyper::{IpfsClient as ApiIpfsClient, TryFromUri};

pub struct IpfsConfig<'a> {
	pub rpc_replicas: Vec<&'a str>,
	pub failure_retry: u8,
}

impl<'a> From<&'a Config> for IpfsConfig<'a> {
	fn from(config: &'a Config) -> Self {
		IpfsConfig { rpc_replicas: config.rpc_replicas(), failure_retry: config.failure_retry }
	}
}

pub struct IpfsClientBuilder<'a> {
	config: IpfsConfig<'a>,
}

const MAX_REPLICAS: usize = 10;

#[async_trait]
impl<'a> ClientBuilder<'a, IpfsClient> for IpfsClientBuilder<'a> {
	fn from_config(config: &'a Config) -> Self {
		let config = IpfsConfig::from(config);
		Self { config }
	}

	async fn build(self) -> IpfsClient {
		let replicas: Result<Vec<ApiIpfsClient>, _> = self
			.config
			.rpc_replicas
			.into_iter()
			.map(|url| ApiIpfsClient::from_str(url))
			.collect();

		match replicas {
			Ok(replicas) => {
				if replicas.is_empty() {
					panic!("No replicas provided");
				}

				if replicas.len() > MAX_REPLICAS {
					panic!("Too many replicas provided");
				}
				IpfsClient::new(replicas, self.config.failure_retry)
			},
			Err(e) => panic!("Failed to create IPFS client: {}", e),
		}
	}
}
