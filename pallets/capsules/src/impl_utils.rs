use crate::{
	capsule::{CapsuleIdFor, CapsuleMetadataOf, Status},
	container::ContainerIdOf,
	AppIdFor, Approval, CapsuleClearCursors, CapsuleCursorsOf, CapsuleItems, Capsules, Config,
	DeletionCompletion, Error, IdComputation, OwnersWaitingApprovals, Ownership, Pallet,
};
use codec::Encode;
use common_types::Accounts;
use frame_support::{ensure, storage::KeyLenOf};
use sp_core::{Get, Hasher};
use sp_runtime::{BoundedVec, DispatchError, DispatchResult};

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
		ensure!(capsule.status == Status::Live, Error::<T>::IncorrectCapsuleStatus);
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

	// Utility for the OwnersWaitingApprovals Cursor
	// Returns wether is has completed the deletion based on `maybe_cursor`.
	// If None then it has completed the clearing
	pub fn modify_cursors_for_approvals(
		capsule_id: &CapsuleIdFor<T>,
		maybe_cursors: Option<&mut CapsuleCursorsOf<T>>,
		maybe_cursor: Option<Vec<u8>>,
	) -> bool {
		if let Some(cursor) = maybe_cursor {
			let ownersip_approvals_cursor = BoundedVec::truncate_from(cursor);
			if let Some(cursors) = maybe_cursors {
				cursors.0 = Some(ownersip_approvals_cursor);
			} else {
				let cursors: CapsuleCursorsOf<T> =
					(Some(ownersip_approvals_cursor), Option::default(), Option::default());
				CapsuleClearCursors::<T>::insert(capsule_id, cursors);
			}
			false
		} else {
			true
		}
	}

	// Utility for the CapsuleFollowers Cursor
	// Returns wether is has completed the deletion based on `maybe_cursor`.
	// If None then it has completed the clearing
	pub fn modify_cursors_for_followers(
		capsule_id: &CapsuleIdFor<T>,
		maybe_cursors: Option<&mut CapsuleCursorsOf<T>>,
		maybe_cursor: Option<Vec<u8>>,
	) -> bool {
		if let Some(cursor) = maybe_cursor {
			let followers_cursor = BoundedVec::truncate_from(cursor);
			if let Some(cursors) = maybe_cursors {
				cursors.1 = Some(followers_cursor);
			} else {
				let cursors: CapsuleCursorsOf<T> =
					(Option::default(), Some(followers_cursor), Option::default());
				CapsuleClearCursors::<T>::insert(capsule_id, cursors);
			}
			false
		} else {
			true
		}
	}

	// Utility for the CapsuleContainers Cursor
	// Returns wether is has completed the deletion based on `maybe_cursor`.
	// If None then it has completed the clearing
	pub fn modify_cursors_for_capsule_containers(
		capsule_id: &CapsuleIdFor<T>,
		maybe_cursors: Option<&mut CapsuleCursorsOf<T>>,
		maybe_cursor: Option<Vec<u8>>,
	) -> bool {
		if let Some(cursor) = maybe_cursor {
			let capsule_containers_cursor = BoundedVec::truncate_from(cursor);
			if let Some(cursors) = maybe_cursors {
				cursors.2 = Some(capsule_containers_cursor);
			} else {
				let cursors: CapsuleCursorsOf<T> =
					(Option::default(), Option::default(), Some(capsule_containers_cursor));
				CapsuleClearCursors::<T>::insert(capsule_id, cursors);
			}
			false
		} else {
			true
		}
	}

	pub fn try_transition_second_destroying_stage(
		capsule: &mut CapsuleMetadataOf<T>,
		completion: &DeletionCompletion,
	) {
		if completion
			== &(DeletionCompletion {
				ownership_approvals: true,
				followers: true,
				container_keys: true,
			}) {
			capsule.status = Status::CapsuleContainersDeletion
		}
	}
}
