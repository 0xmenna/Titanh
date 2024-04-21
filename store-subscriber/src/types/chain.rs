use substrate_api_client::{
	ac_primitives::{Config, DefaultRuntimeConfig},
	rpc::JsonrpseeClient,
	Api,
};

pub type ValidatorKeyPair = <DefaultRuntimeConfig as Config>::CryptoKey;
pub type DefaultApi = Api<DefaultRuntimeConfig, JsonrpseeClient>;
