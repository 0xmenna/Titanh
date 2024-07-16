use pallet_capsules::CapsuleMetadataOf;
use substrate_api_client::{
	ac_primitives::{Config, DefaultRuntimeConfig},
	rpc::JsonrpseeClient,
	Api,
};
use titanh_runtime::Runtime as TitanhRuntimeConfig;
pub use titanh_runtime::RuntimeEvent;

/// The keypair used for the Substrate API Client.
pub type ValidatorKeyPair = <DefaultRuntimeConfig as Config>::CryptoKey;
/// The Substrate API Client.
pub type DefaultApi = Api<DefaultRuntimeConfig, JsonrpseeClient>;

pub type CapsuleMetadata = CapsuleMetadataOf<TitanhRuntimeConfig>;
