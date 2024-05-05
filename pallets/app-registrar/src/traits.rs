use crate::{AppPermissions, Config, Pallet};
use codec::MaxEncodedLen;
use frame_support::Parameter;
use sp_runtime::traits::{MaybeSerializeDeserialize, Member};

pub trait PermissionsApp<AccountId> {
	type AppId: Member + Parameter + Clone + MaybeSerializeDeserialize + MaxEncodedLen;

	fn has_account_permissions(account: &AccountId, app: Self::AppId) -> bool;
}

impl<T: Config> PermissionsApp<T::AccountId> for Pallet<T> {
	type AppId = T::AppId;

	fn has_account_permissions(account: &T::AccountId, app: Self::AppId) -> bool {
		AppPermissions::<T>::get(app, account)
	}
}
