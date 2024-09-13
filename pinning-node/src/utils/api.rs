use crate::types::chain::ValidatorKeyPair;
use anyhow::{anyhow, Result};
use sp_core::Pair;
use subxt::{tx::PairSigner, OnlineClient, SubstrateConfig};
use url::Url;

/// Substrate api with a default configuration
#[derive(Clone)]
pub struct SubstrateApi {
	api: OnlineClient<SubstrateConfig>,
	signer: PairSigner<SubstrateConfig, ValidatorKeyPair>,
}

impl SubstrateApi {
	pub fn api(&self) -> OnlineClient<SubstrateConfig> {
		self.api.clone()
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
	rpc_url: RpcEndpoint,
	signer: PairSigner<SubstrateConfig, ValidatorKeyPair>,
}

/// Builder for the Substrate API
#[derive(Default)]
pub struct SubstrateApiBuilder<T>(T);

impl SubstrateApiBuilder<ApiNotInitialized> {
	pub fn rpc_url(self, url: &str) -> Result<SubstrateApiBuilder<RpcEndpoint>> {
		let url = Url::parse(url).map_err(|_| anyhow!("Invalid url: {:?}", url))?;

		Ok(SubstrateApiBuilder(RpcEndpoint(url)))
	}

	pub fn default_rpc_url(self) -> SubstrateApiBuilder<RpcEndpoint> {
		SubstrateApiBuilder(RpcEndpoint(Url::parse("ws://127.0.0.1:9944").unwrap()))
	}
}

impl SubstrateApiBuilder<RpcEndpoint> {
	pub fn keyring_material(
		self,
		phrase: &str,
		password: Option<&str>,
	) -> Result<SubstrateApiBuilder<ApiReady>> {
		// Derive the key pair from the seed phrase (mnemonic)
		let key_pair = ValidatorKeyPair::from_string(phrase, None)
			.map_err(|_| anyhow!("Error retrieving keypair"))?;

		// Create a signer using the key pair
		let signer: PairSigner<SubstrateConfig, ValidatorKeyPair> = PairSigner::new(key_pair);

		Ok(SubstrateApiBuilder(ApiReady { rpc_url: self.0, signer }))
	}
}

impl SubstrateApiBuilder<ApiReady> {
	pub async fn build(self) -> Result<SubstrateApi> {
		let api = OnlineClient::<SubstrateConfig>::from_url(self.0.rpc_url.as_str())
			.await
			.map_err(|_| anyhow!("Invalid endpoint"))?;

		Ok(SubstrateApi { api, signer: self.0.signer })
	}
}
