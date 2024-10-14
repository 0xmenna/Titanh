use crate::{AppPermissions, Config, CurrentAppId, Pallet, PermissionState};
use codec::MaxEncodedLen;
use frame_support::Parameter;
use sp_runtime::traits::{MaybeSerializeDeserialize, Member};

pub trait PermissionsApp<AccountId> {
    type AppId: Member + Parameter + Clone + MaybeSerializeDeserialize + MaxEncodedLen;

    fn has_account_permissions(account: &AccountId, app: Self::AppId) -> bool;

    fn current_app_id() -> Self::AppId;
}

impl<T: Config> PermissionsApp<T::AccountId> for Pallet<T> {
    type AppId = T::AppId;

    fn has_account_permissions(account: &T::AccountId, app: Self::AppId) -> bool {
        AppPermissions::<T>::get(app, account)
            .map(|permission_state| permission_state == PermissionState::Active)
            .unwrap_or_default()
    }

    fn current_app_id() -> Self::AppId {
        CurrentAppId::<T>::get()
    }
}
