use super::substrate_storage::{StorageKeyBuilder, StorageKeyData};
use crate::types::chain::{DefaultApi, ValidatorKeyPair};
use anyhow::{anyhow, Result};
use codec::Decode;
use sp_core::{sr25519::Pair as CryptoPair, storage::StorageKey, Pair};
use substrate_api_client::{
	ac_primitives::DefaultRuntimeConfig, rpc::JsonrpseeClient, Api, GetStorage,
};
use url::Url;

/// Substrate api with a default configuration
#[derive(Clone)]
pub struct SubstrateApi(DefaultApi);

impl SubstrateApi {
	/// Create a new storage key builder for a module and storage item
	pub fn storage_key_builder(
		&self,
		module_name: &str,
		storage_name: &str,
	) -> StorageKeyBuilder<StorageKeyData> {
		StorageKeyBuilder::default()
			.module_name(module_name)
			.storage_name(storage_name)
			.create_storage_items()
	}

	/// Get a storage value by key
	pub async fn get_storage_by_key<T: Decode>(&self, key: StorageKey) -> Result<T> {
		let value: T = self
			.0
			.get_storage_by_key(key.clone(), None)
			.await
			.map_err(|_| anyhow!("Error decoding value for key: {:?}", key))?
			.unwrap();

		Ok(value)
	}
}

#[derive(Default)]
pub struct ApiNotInitialzed;
pub struct RpcEndpoint(Url);

impl RpcEndpoint {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

pub struct ApiReady {
	rpc_url: RpcEndpoint,
	keypair: ValidatorKeyPair,
}

/// Builder for the Substrate API
#[derive(Default)]
pub struct SubstrateApiBuilder<T>(T);

impl SubstrateApiBuilder<ApiNotInitialzed> {
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
		let keypair = CryptoPair::from_phrase(phrase, password)
			.map_err(|_| anyhow!("Error retrieving keypair"))?
			.0;

		Ok(SubstrateApiBuilder(ApiReady { rpc_url: self.0, keypair }))
	}
}

impl SubstrateApiBuilder<ApiReady> {
	pub async fn build(self) -> Result<SubstrateApi> {
		// Initialize the api
		let client = JsonrpseeClient::new(self.0.rpc_url.as_str())
			.await
			.map_err(|_| anyhow!("Client url error"))?;

		let mut api = Api::<DefaultRuntimeConfig, _>::new(client)
			.await
			.map_err(|_| anyhow!("Runtime configuration error"))?;

		api.set_signer(self.0.keypair.into());

		Ok(SubstrateApi(api))
	}
}
