use super::ContainerIdOf;
use crate::{
	capsule::CapsuleIdFor,
	container::{ContainerMetadata, ContainerStatus},
	AppData, AppIdFor, Approval, Config, Container, ContainerDetails, Error, Event,
	FollowersStatus, IdComputation, Pallet,
};
use common_types::{BoundedString, EncodedData};
use frame_support::ensure;
use pallet_app_registrar::PermissionsApp;
use sp_runtime::{traits::Saturating, DispatchResult};
use sp_std::vec::Vec;

/// Container related logic
impl<T: Config> Pallet<T> {
	pub fn create_container_from(
		who: T::AccountId,
		app_id: AppIdFor<T>,
		maybe_other_owner: Option<T::AccountId>,
		container_metadata: Vec<u8>,
	) -> DispatchResult {
		ensure!(
			T::Permissions::has_account_permissions(&who, app_id.clone()),
			Error::<T>::AppPermissionDenied
		);

		let container_id =
			Self::compute_id(app_id.clone(), container_metadata.clone(), IdComputation::Container);
		ensure!(!Self::container_exists(&container_id), Error::<T>::InvalidContainerId);

		let ownership = Self::ownership_from(who, maybe_other_owner);
		let owners = Self::create_owners_from(&ownership, &container_id, Approval::Container);

		ContainerDetails::<T>::insert(
			&container_id,
			ContainerMetadata {
				status: ContainerStatus::default(),
				// There are no capsules attached
				size: 0,
				owners: owners.try_into().map_err(|_| Error::<T>::TooManyOwners)?,
				app_data: AppData {
					app_id: app_id.clone(),
					data: EncodedData::from_slice(&container_metadata)
						.map_err(|_| Error::<T>::BadAppData)?,
				},
			},
		);

		Self::deposit_event(Event::<T>::ContainerCreated {
			container_id,
			app_id,
			app_data: container_metadata,
			ownership,
		});

		Ok(())
	}

	pub fn approve_container_ownership_from(
		who: T::AccountId,
		container_id: ContainerIdOf<T>,
	) -> DispatchResult {
		let mut container =
			ContainerDetails::<T>::get(&container_id).ok_or(Error::<T>::InvalidContainerId)?;
		// Try to approve a container waiting approval, if any
		Self::try_approve_ownership(&who, &container_id, Approval::Container)?;
		// Try to add the owner to container owners, if it does not exceeds the vector bounds
		Self::try_add_owner(&who, &mut container.owners)?;

		// Emit Event
		Self::deposit_event(Event::<T>::OwnershipApproved {
			id: container_id,
			who,
			approval: Approval::Container,
		});

		Ok(())
	}

	pub fn share_container_ownership_from(
		who: T::AccountId,
		container_id: ContainerIdOf<T>,
		other_owner: T::AccountId,
	) -> DispatchResult {
		// Obtain the container from the owner `who`
		// Dispatches an error if `who` is not an owner of the container
		let container =
			ContainerDetails::<T>::get(container_id).ok_or(Error::<T>::InvalidContainerId)?;
		ensure!(container.owners.binary_search(&who).is_ok(), Error::<T>::BadOriginForOwnership);

		Self::try_share_ownership(
			&container_id,
			&other_owner,
			container.owners.to_vec(),
			Approval::Container,
		)?;

		// Emit Event
		Self::deposit_event(Event::<T>::SharedOwnership {
			id: container_id,
			who: other_owner,
			waiting_approval: Approval::Container,
		});

		Ok(())
	}

	/// Attach a capsule to a container. The capsule will be identified in the container within `key`.
	pub fn attach_capsule_to_container_from(
		who: T::AccountId,
		container_id: ContainerIdOf<T>,
		key: Vec<u8>,
		capsule_id: CapsuleIdFor<T>,
	) -> DispatchResult {
		let (mut container, _) = Self::container_from_maybe_owner(&who, &container_id)?;
		Self::capsule_from_owner(&who, &capsule_id)?;

		// Check that a capsule identified by `key` is not already defined within the given container
		let key = BoundedString::from_vec(key).map_err(|_| Error::<T>::BadKeyFormat)?;
		ensure!(Container::<T>::get(&container_id, &key).is_none(), Error::<T>::BadKey);

		// Attach the capsule to the container using `key`
		Container::<T>::insert(&container_id, &key, capsule_id.clone());
		container.size.saturating_inc();

		Self::deposit_event(Event::<T>::CapsuleAttached {
			container_id,
			key: key.to_vec(),
			capsule_id,
		});

		Ok(())
	}

	pub fn change_container_status_from(
		who: T::AccountId,
		container_id: ContainerIdOf<T>,
		status: ContainerStatus,
	) -> DispatchResult {
		let mut container =
			ContainerDetails::<T>::get(&container_id).ok_or(Error::<T>::InvalidContainerId)?;
		ensure!(container.owners.binary_search(&who).is_ok(), Error::<T>::BadOriginForOwnership);
		container.set_status(status.clone());

		Self::deposit_event(Event::<T>::ContainerStatusChanged { container_id, status });

		Ok(())
	}

	/// Detach a capsule identified by `key` from a container.
	pub fn detach_capsule_from_container(
		who: T::AccountId,
		container_id: ContainerIdOf<T>,
		key: Vec<u8>,
	) -> DispatchResult {
		let (mut container, requires_ownership) =
			Self::container_from_maybe_owner(&who, &container_id)?;
		// Check that a capsule identified by `key` is not already defined within the given container
		let key = BoundedString::from_vec(key).map_err(|_| Error::<T>::BadKeyFormat)?;
		let capsule_id = Container::<T>::get(&container_id, &key).ok_or(Error::<T>::BadKey)?;
		// If te container doesn't require owneship, than it means is in a public state, perhaps to detach a capsule from a container we check if `who` is the owner of the capsule
		// Else, we allow container owners to detach all capsules
		if !requires_ownership {
			Self::capsule_from_owner(&who, &capsule_id)?;
		}
		// Detach the capsule from the container using `key`
		Container::<T>::remove(&container_id, &key);
		container.size.saturating_dec();

		Self::deposit_event(Event::<T>::CapsuleDetached {
			container_id,
			key: key.to_vec(),
			capsule_id,
		});

		Ok(())
	}
}
