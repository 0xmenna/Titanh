use crate::{
	capsule::{CapsuleIdFor, CapsuleMetadataOf, Status},
	container::ContainerIdOf,
	AppIdFor, Approval, Capsules, Config, Error, IdComputation, OwnersWaitingApprovals, Ownership,
	Pallet,
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

	/// Computes the capsule/container identifier
	// To avoid id duplications for capsules and containers with the same `metadata` and `app_id`, we add static prefixes.
	// Given a capsule/container, every "app" can have its own new capsule/container, as long as the metadata is different.
	// Different apps can have the same metadata for their capsules/containers.
	pub fn compute_id(
		app_id: AppIdFor<T>,
		metadata: Vec<u8>,
		what: IdComputation,
	) -> CapsuleIdFor<T> {
		let mut data = Vec::new();
		match what {
			IdComputation::Capsule => data.extend_from_slice(T::CapsuleIdPrefix::get()),
			IdComputation::Container => data.extend_from_slice(T::ContainerIdPrefix::get()),
		}

		data.extend_from_slice(&app_id.encode());
		data.extend_from_slice(&metadata);

		T::Hashing::hash(&data[..])
	}

	pub fn capsule_exists(capsule_id: &CapsuleIdFor<T>) -> bool {
		Capsules::<T>::get(capsule_id).is_some()
	}

	pub fn try_approve_capsule_ownership(
		who: &T::AccountId,
		capsule_id: &CapsuleIdFor<T>,
	) -> DispatchResult {
		OwnersWaitingApprovals::<T>::get(capsule_id, who)
			.filter(|approval| approval == &Approval::Capsule)
			.ok_or(Error::<T>::NoWaitingApproval)?;

		OwnersWaitingApprovals::<T>::remove(capsule_id, who);
		Ok(())
	}

	pub fn try_approve_container_ownership(
		who: &T::AccountId,
		container_id: &ContainerIdOf<T>,
	) -> DispatchResult {
		OwnersWaitingApprovals::<T>::get(container_id, who)
			.filter(|approval| approval == &Approval::Container)
			.ok_or(Error::<T>::NoWaitingApproval)?;

		OwnersWaitingApprovals::<T>::remove(container_id, who);
		Ok(())
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

	pub fn ensure_capsule_liveness(capsule: &CapsuleMetadataOf<T>) -> DispatchResult {
		ensure!(capsule.status != Status::Destroying, Error::<T>::DestroyingCapsule);
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
