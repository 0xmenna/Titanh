use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::*;
use frame_system::Config as SystemConfig;
use pallet_app_registrar::PermissionsApp;
use scale_info::TypeInfo;
use sp_core::Get;
use sp_std::prelude::*;

/// An application specific identifier
pub type AppIdFor<T> =
	<<T as Config>::Permissions as PermissionsApp<<T as SystemConfig>::AccountId>>::AppId;

/// Account balance
pub type BalanceOf<T> = <T as Config>::Balance;

#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(S))]
pub struct AppData<AppId, S: Get<u32>> {
	/// An application specific identifier
	pub app_id: AppId,
	/// The app scale encoded data
	pub data: EncodedData<S>,
}

/// The type of capsule follower
#[derive(Encode, Decode, MaxEncodedLen, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub enum Follower {
	#[default]
	Basic,
	Privileged,
}

/// What kind of followers are allowed for a given capsule
#[derive(Encode, Decode, MaxEncodedLen, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum FollowersStatus {
	#[default]
	None,
	Basic,
	Privileged,
	All,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug, TypeInfo)]
pub struct LetsTry {
	abc: u32,
}
