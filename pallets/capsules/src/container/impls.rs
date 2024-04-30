use common_types::EncodedData;
use frame_support::ensure;
use pallet_app_registrar::PermissionsApp;
use sp_runtime::DispatchResult;

use crate::{
	container::ContainerMetadata, AppData, AppIdFor, Approval, Config, ContainerDetails, Error,
	FollowersStatus, IdComputation, Pallet,
};

/// Container related logic
impl<T: Config> Pallet<T> {
	pub fn create_container_from(
		who: T::AccountId,
		app_id: AppIdFor<T>,
		maybe_other_owner: Option<T::AccountId>,
		followers_status: FollowersStatus,
		app_data: Vec<u8>,
	) -> DispatchResult {
		ensure!(
			T::Permissions::has_account_permissions(&who, app_id.clone()),
			Error::<T>::AppPermissionDenied
		);

		let ownership = Self::ownership_from(who, maybe_other_owner);

		let container_id =
			Self::compute_id(app_id.clone(), app_data.clone(), IdComputation::Container);

		ensure!(!Self::capsule_exists(&container_id), Error::<T>::InvalidContainerId);

		let owners = Self::create_owners_from(ownership, &container_id, Approval::Container);

		ContainerDetails::<T>::insert(
			&container_id,
			ContainerMetadata {
				size: 0,
				owners: owners.try_into().map_err(|_| Error::<T>::TooManyOwners)?,
				followers_status,
				app_data: AppData {
					app_id,
					data: EncodedData::from_slice(&app_data).map_err(|_| Error::<T>::BadAppData)?,
				},
			},
		);

		Self::deposit_event(Event::<T>::ContainerUploaded {
			container_id,
			app_id,
			cid: capsule_metadata.cid,
			size: capsule_metadata.size,
			app_data: capsule_metadata.app_data.data.to_vec(),
			ownership,
		});

		Ok(())
	}
}
