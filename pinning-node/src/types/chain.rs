use codec::{Decode, Encode, MaxEncodedLen};
use pallet_app_registrar::Event;
use pallet_capsules::CapsuleMetadataOf;
use sp_core::RuntimeDebug;
use sp_runtime::traits::Member;
use substrate_api_client::{
	ac_primitives::{Config, DefaultRuntimeConfig},
	rpc::JsonrpseeClient,
	Api,
};
pub use titanh_runtime::Runtime as TitanhRuntimeConfig;
pub use titanh_runtime::RuntimeEvent;

pub type ValidatorKeyPair = sp_core::sr25519::Pair;

/// The Substrate API Client.
pub type DefaultApi = Api<DefaultRuntimeConfig, JsonrpseeClient>;

pub type CapsuleMetadata = CapsuleMetadataOf<TitanhRuntimeConfig>;

pub type AppRegistrarEvents = Event<TitanhRuntimeConfig>;
