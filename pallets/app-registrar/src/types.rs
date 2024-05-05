use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Encode, Decode, MaxEncodedLen, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum AppSubscriptionStatus {
	//Anyone can subscribe to the app
	Anyone,
	#[default]
	//The owner selects the users permissions
	SelectedByOwner,
}

#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct AppDetails<AccountId> {
	pub owner: AccountId,
	pub status: AppSubscriptionStatus,
}
