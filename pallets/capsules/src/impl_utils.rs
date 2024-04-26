use crate::{
	capsule::{CapsuleIdFor, CapsuleMetadataOf},
	container::ContainerIdOf,
	AppIdFor, Approval, Capsules, Config, Error, OwnersWaitingApprovals, Ownership, Pallet,
};
use codec::Encode;
use common_types::Accounts;
use frame_support::ensure;
use sp_core::{Get, Hasher};
use sp_runtime::{DispatchError, DispatchResult};

impl<T: Config> Pallet<T> {
	pub fn ownership_from(
		who: T::AccountId,
		maybe_account: Option<T::AccountId>,
	) -> Ownership<T::AccountId> {
		maybe_account
			.map(|owner| Ownership::Other(owner))
			.unwrap_or_else(|| Ownership::Signer(who))
	}

	pub fn compute_id(app_id: AppIdFor<T>, metadata: Vec<u8>) -> CapsuleIdFor<T> {
		let mut data = Vec::new();
		data.push(app_id.encode());
		data.push(metadata);

		T::Hashing::hash(&data.concat()[..])
	}

	pub fn capsule_exists(capsule_id: &CapsuleIdFor<T>) -> bool {
		Capsules::<T>::get(capsule_id).is_some()
	}

	pub fn try_approve_capsule_ownership(
		who: &T::AccountId,
		capsule_id: &CapsuleIdFor<T>,
	) -> DispatchResult {
		if OwnersWaitingApprovals::<T>::get(who, capsule_id) == Approval::Capsule {
			OwnersWaitingApprovals::<T>::insert(who, capsule_id, Approval::None);
			Ok(())
		} else {
			Err(Error::<T>::NoWaitingApproval.into())
		}
	}

	pub fn try_approve_container_ownership(
		who: &T::AccountId,
		container_id: &ContainerIdOf<T>,
	) -> DispatchResult {
		if OwnersWaitingApprovals::<T>::get(who, container_id) == Approval::Container {
			OwnersWaitingApprovals::<T>::insert(who, container_id, Approval::None);
			Ok(())
		} else {
			Err(Error::<T>::NoWaitingApproval.into())
		}
	}

	pub fn try_add_owner<S: Get<u32>>(
		who: &T::AccountId,
		owners: &mut Accounts<T::AccountId, S>,
	) -> DispatchResult {
		// Get the position of `who` in the owners' list
		// Safe note: `who` can never be in that list
		let idx = owners.binary_search(&who).expect_err("The account cannot be an owner");
		owners.try_insert(idx, who.clone()).map_err(|_| Error::<T>::TooManyOwners)?;

		Ok(())
	}

	pub fn capsule_from_owner(
		who: &T::AccountId,
		capsule_id: &CapsuleIdFor<T>,
	) -> Result<CapsuleMetadataOf<T>, DispatchError> {
		if let Some(capsule) = Capsules::<T>::get(&capsule_id) {
			// check if `who` is an owner of the capsule
			ensure!(capsule.owners.binary_search(&who).is_ok(), Error::<T>::BadOriginForOwnership);

			Ok(capsule)
		} else {
			Err(Error::<T>::InvalidCapsuleId.into())
		}
	}
}
