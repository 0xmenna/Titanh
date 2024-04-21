use crate::types::chain::{DefaultApi, ValidatorKeyPair};
use sp_core::{sr25519::Pair as CryptoPair, Pair};
use substrate_api_client::{ac_primitives::DefaultRuntimeConfig, rpc::JsonrpseeClient, Api};
use url::Url;

pub enum Error {
    InvalidUrl,
    InvalidKeyringMaterial,
    ClientUrlError,
    ApiRuntimeConfigError,
}

/// Substrate api with a default configuration
#[derive(Clone)]
pub struct SubstrateApi(DefaultApi);

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

#[derive(Default)]
pub struct SubstrateApiBuilder<T>(T);

impl SubstrateApiBuilder<ApiNotInitialzed> {
    pub fn rpc_url(self, url: &str) -> Result<SubstrateApiBuilder<RpcEndpoint>, Error> {
        let url = Url::parse(url).map_err(|_| Error::InvalidUrl)?;

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
    ) -> Result<SubstrateApiBuilder<ApiReady>, Error> {
        let keypair = CryptoPair::from_phrase(phrase, password)
            .map_err(|_| Error::InvalidKeyringMaterial)?
            .0;

        Ok(SubstrateApiBuilder(ApiReady {
            rpc_url: self.0,
            keypair,
        }))
    }
}

impl SubstrateApiBuilder<ApiReady> {
    pub async fn build(self) -> Result<SubstrateApi, Error> {
        // Initialize the api
        let client = JsonrpseeClient::new(self.0.rpc_url.as_str())
            .await
            .map_err(|_| Error::ClientUrlError)?;

        let mut api = Api::<DefaultRuntimeConfig, _>::new(client)
            .await
            .map_err(|_| Error::ApiRuntimeConfigError)?;

        api.set_signer(self.0.keypair.into());

        Ok(SubstrateApi(api))
    }
}
