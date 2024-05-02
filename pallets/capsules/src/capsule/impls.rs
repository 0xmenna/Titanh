use super::{CapsuleIdFor, CapsuleMetaBuilder, CapsuleUploadData};
use crate::{
	capsule::Status, AppIdFor, Approval, CapsuleClearCursors, CapsuleContainers, CapsuleFollowers,
	CapsuleItems, Capsules, Config, Container, Error, Event, Follower, FollowersStatus,
	IdComputation, OwnersWaitingApprovals, Ownership, Pallet,
};
use common_types::{BlockNumberFor, CidFor, ContentSize};
use frame_support::ensure;
use pallet_app_registrar::PermissionsApp;
use sp_runtime::DispatchResult;

/// Capsule related logic
impl<T: Config> Pallet<T> {
	pub fn upload_capsule_from(
		who: T::AccountId,
		app: AppIdFor<T>,
		maybe_other_owner: Option<T::AccountId>,
		capsule: CapsuleUploadData<CidFor<T>, BlockNumberFor<T>>,
	) -> DispatchResult {
		ensure!(
			T::Permissions::has_account_permissions(&who, app.clone()),
			Error::<T>::AppPermissionDenied
		);
		// If no owner is provided as input, then the signer automatically becomes the owner.
		// Otherwise the ownership is passed to the input account
		let ownership = Self::ownership_from(who, maybe_other_owner);
		// capsule id = hash(prefix + app + encoded_metadata)
		let capsule_id =
			Self::compute_id(app.clone(), capsule.encoded_metadata.clone(), IdComputation::Capsule);

		Self::upload_capsule_data(capsule_id, app, ownership, capsule)
	}

	pub fn approve_capsule_ownership_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
	) -> DispatchResult {
		// We allow the approval even if the capsule is destroying, in this way we do not have to handle the deletion of the approval account.
		// Moreover, the approval account might be in charge of completing the deletion.
		let mut capsule = Capsules::<T>::get(&capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;
		// Try to approve a capsule waiting approval, if any
		Self::try_approve_ownership(&who, &capsule_id, Approval::Capsule)?;
		// Try to add the owner to capsule owners, if it does not exceeds the vector bounds
		Self::try_add_owner(&who, &mut capsule.owners)?;

		// Emit Event
		Self::deposit_event(Event::<T>::OwnershipApproved {
			id: capsule_id,
			who,
			approval: Approval::Capsule,
		});

		Ok(())
	}

	pub fn share_capsule_ownership_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
		other_owner: T::AccountId,
	) -> DispatchResult {
		// Obtain the capsule from the owner `who`
		// Dispatches an error if `who` is not an owner of the capsule
		let capsule = Self::capsule_from_owner(&who, &capsule_id)?;
		Self::ensure_capsule_liveness(&capsule)?;

		Self::try_share_ownership(
			&capsule_id,
			&other_owner,
			capsule.owners.to_vec(),
			Approval::Capsule,
		)?;

		// Emit Event
		Self::deposit_event(Event::<T>::SharedOwnership { id: capsule_id, who: other_owner, waiting_approval: Approval::Capsule });

		Ok(())
	}

	pub fn set_capsule_followers_status_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
		followers_status: FollowersStatus,
	) -> DispatchResult {
		let mut capsule = Self::capsule_from_owner(&who, &capsule_id)?;
		Self::ensure_capsule_liveness(&capsule)?;
		capsule.followers_status = followers_status.clone();

		// Emit event
		Self::deposit_event(Event::<T>::CapsuleFollowersStatusChanged {
			capsule_id,
			status: followers_status,
		});

		Ok(())
	}

	pub fn follow_capsule_from(who: T::AccountId, capsule_id: CapsuleIdFor<T>) -> DispatchResult {
		let capsule = Capsules::<T>::get(&capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;
		Self::ensure_capsule_liveness(&capsule)?;
		// check the followers status correspondence
		ensure!(
			capsule.followers_status == FollowersStatus::Basic
				|| capsule.followers_status == FollowersStatus::All,
			Error::<T>::BadFollowersStatus
		);
		// check that `who` is not already a follower
		ensure!(
			CapsuleFollowers::<T>::get(&capsule_id, &who).is_none(),
			Error::<T>::AlreadyFollower
		);
		CapsuleFollowers::<T>::insert(&capsule_id, &who, Follower::Basic);

		// Emit event
		Self::deposit_event(Event::<T>::CapsuleFollowed { capsule_id, follower: who });

		Ok(())
	}

	pub fn update_capsule_content_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
		cid: CidFor<T>,
		size: ContentSize,
	) -> DispatchResult {
		let mut capsule = Self::capsule_from_owner(&who, &capsule_id)?;
		Self::ensure_capsule_liveness(&capsule)?;
		// change the capsule cid and size
		capsule.cid = cid;
		capsule.size = size;

		Self::deposit_event(Event::<T>::CapsuleContentChanged { capsule_id, cid, size });

		Ok(())
	}

	pub fn extend_ending_retention_block_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
		at_block: BlockNumberFor<T>,
	) -> DispatchResult {
		let mut capsule = Self::capsule_from_owner(&who, &capsule_id)?;
		Self::ensure_capsule_liveness(&capsule)?;
		ensure!(at_block > capsule.ending_retention_block, Error::<T>::BadBlockNumber);
		capsule.ending_retention_block = at_block;

		Self::deposit_event(Event::<T>::CapsuleEndingRetentionBlockExtended {
			capsule_id,
			at_block,
		});

		Ok(())
	}

	pub fn add_priviledged_follower_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
		follower: T::AccountId,
	) -> DispatchResult {
		let capsule = Self::capsule_from_owner(&who, &capsule_id)?;
		Self::ensure_capsule_liveness(&capsule)?;
		// check the followers status correspondence
		ensure!(
			capsule.followers_status == FollowersStatus::Privileged
				|| capsule.followers_status == FollowersStatus::All,
			Error::<T>::BadFollowersStatus
		);
		// check that `follower` is not already a priviledged follower or is in a waiting approval state
		ensure!(
			CapsuleFollowers::<T>::get(&capsule_id, &follower).unwrap_or_default()
				== Follower::Basic,
			Error::<T>::AlreadyFollower
		);
		CapsuleFollowers::<T>::insert(
			&capsule_id,
			&follower,
			Follower::WaitingApprovalForPrivileged,
		);

		// Emit event
		Self::deposit_event(Event::<T>::PrivilegedFollowerWaitingToApprove {
			capsule_id,
			who: follower,
		});

		Ok(())
	}

	pub fn approve_privileged_follow_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
	) -> DispatchResult {
		if let Some(capsule) = Capsules::<T>::get(&capsule_id) {
			Self::ensure_capsule_liveness(&capsule)?;
			// check that `who` is in a waiting approval state
			ensure!(
				CapsuleFollowers::<T>::get(&capsule_id, &who).unwrap_or_default()
					== Follower::WaitingApprovalForPrivileged,
				Error::<T>::NoWaitingApproval
			);
			CapsuleFollowers::<T>::insert(&capsule_id, &who, Follower::Privileged);

			// Emit event
			Self::deposit_event(Event::<T>::PrivilegedFollowApproved { capsule_id, who });

			Ok(())
		} else {
			Err(Error::<T>::InvalidCapsuleId.into())
		}
	}

	fn upload_capsule_data(
		capsule_id: CapsuleIdFor<T>,
		app_id: AppIdFor<T>,
		ownership: Ownership<T::AccountId>,
		metadata: CapsuleUploadData<CidFor<T>, BlockNumberFor<T>>,
	) -> DispatchResult {
		ensure!(!Self::capsule_exists(&capsule_id), Error::<T>::CapsuleIdAlreadyExists);

		let owners = Self::create_owners_from(&ownership, &capsule_id, Approval::Capsule);

		// Construct storing metadata and insert into storage
		let capsule_metadata = CapsuleMetaBuilder::<T>::new(app_id, owners, metadata).build()?;
		Capsules::<T>::insert(&capsule_id, capsule_metadata.clone());

		// Emit Upload Event
		Self::deposit_event(Event::<T>::CapsuleUploaded {
			id: capsule_id,
			app_id: capsule_metadata.app_data.app_id,
			cid: capsule_metadata.cid,
			size: capsule_metadata.size,
			app_data: capsule_metadata.app_data.data.to_vec(),
			ownership,
			followers_status: capsule_metadata.followers_status,
		});

		Ok(())
	}

	pub fn start_destroy_capsule_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
	) -> DispatchResult {
		let mut capsule = Capsules::<T>::get(&capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;
		assert!(
			capsule.status == Status::Live,
			"The capsule must be live to transition to the first destroying stage"
		);
		// If the retention period has elapsed, anyone is allowed to destroy the capsule.
		// This is to increase the level of decentralization.
		// Else, only an owner is capable to start the deletion of a capsule
		if capsule.ending_retention_block > <frame_system::Pallet<T>>::block_number() {
			ensure!(capsule.owners.binary_search(&who).is_ok(), Error::<T>::BadOriginForOwnership);
		}
		capsule.status = Status::ItemsDeletion(Default::default());

		Ok(())
	}

	pub fn destroy_ownership_approvals_from(
		capsule_id: CapsuleIdFor<T>,
		max: u32,
	) -> DispatchResult {
		let mut capsule = Capsules::<T>::get(capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;

		let mut maybe_cursors = CapsuleClearCursors::<T>::get(&capsule_id);
		let r = OwnersWaitingApprovals::<T>::clear_prefix(
			&capsule_id,
			max,
			maybe_cursors
				.clone()
				.map(|cursors| cursors.0)
				.flatten()
				.as_ref()
				.map(|x| &x[..]),
		);

		let removal_completion =
			Self::modify_cursors_for_approvals(&capsule_id, maybe_cursors.as_mut(), r.maybe_cursor);

		if let Status::ItemsDeletion(mut deletion_completition) = capsule.status.clone() {
			if removal_completion {
				deletion_completition.ownership_approvals = true;
				Self::try_transition_second_destroying_stage(&mut capsule, &deletion_completition);
			}
			Self::deposit_event(Event::<T>::CapsuleItemsDeleted {
				capsule_id,
				removal_completion,
				items: CapsuleItems::WaitingOwnershipApprovals,
			});

			Ok(())
		} else {
			return Err(Error::<T>::IncorrectCapsuleStatus.into());
		}
	}

	pub fn destroy_followers_from(capsule_id: CapsuleIdFor<T>, max: u32) -> DispatchResult {
		let mut capsule = Capsules::<T>::get(capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;

		let mut maybe_cursors = CapsuleClearCursors::<T>::get(&capsule_id);
		let r = CapsuleFollowers::<T>::clear_prefix(
			&capsule_id,
			max,
			maybe_cursors
				.clone()
				.map(|cursors| cursors.1)
				.flatten()
				.as_ref()
				.map(|x| &x[..]),
		);

		let removal_completion =
			Self::modify_cursors_for_followers(&capsule_id, maybe_cursors.as_mut(), r.maybe_cursor);

		if let Status::ItemsDeletion(mut deletion_completition) = capsule.status.clone() {
			if removal_completion {
				deletion_completition.followers = true;
				Self::try_transition_second_destroying_stage(&mut capsule, &deletion_completition);
			}
			Self::deposit_event(Event::<T>::CapsuleItemsDeleted {
				capsule_id,
				removal_completion,
				items: CapsuleItems::Followers,
			});

			Ok(())
		} else {
			return Err(Error::<T>::IncorrectCapsuleStatus.into());
		}
	}

	pub fn destroy_container_keys_of(capsule_id: CapsuleIdFor<T>, max: u32) -> DispatchResult {
		let mut capsule = Capsules::<T>::get(capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;

		let mut removal_completion = true;
		for (i, (container_id, key)) in CapsuleContainers::<T>::iter_prefix(&capsule_id).enumerate()
		{
			if i >= max as usize {
				removal_completion = false;
				break;
			}

			Container::<T>::remove(container_id, key);
		}

		if let Status::ItemsDeletion(mut deletion_completition) = capsule.status.clone() {
			if removal_completion {
				deletion_completition.container_keys = true;
				Self::try_transition_second_destroying_stage(&mut capsule, &deletion_completition);
			}

			Self::deposit_event(Event::<T>::CapsuleItemsDeleted {
				capsule_id,
				removal_completion,
				items: CapsuleItems::KeysInContainers,
			});

			Ok(())
		} else {
			return Err(Error::<T>::IncorrectCapsuleStatus.into());
		}
	}

	pub fn destroy_capsule_containers_from(
		capsule_id: CapsuleIdFor<T>,
		max: u32,
	) -> DispatchResult {
		let mut capsule = Capsules::<T>::get(capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;
		ensure!(
			capsule.status == Status::CapsuleContainersDeletion,
			Error::<T>::IncorrectCapsuleStatus
		);

		let mut maybe_cursors = CapsuleClearCursors::<T>::get(&capsule_id);
		let r = CapsuleFollowers::<T>::clear_prefix(
			&capsule_id,
			max,
			maybe_cursors
				.clone()
				.map(|cursors| cursors.2)
				.flatten()
				.as_ref()
				.map(|x| &x[..]),
		);

		let removal_completion = Self::modify_cursors_for_capsule_containers(
			&capsule_id,
			maybe_cursors.as_mut(),
			r.maybe_cursor,
		);

		if removal_completion {
			capsule.status = Status::FinalDeletion
		}

		Self::deposit_event(Event::<T>::CapsuleContainersDeleted {
			capsule_id,
			removal_completion,
		});

		Ok(())
	}

	pub fn finish_destroy_capsule_from(capsule_id: CapsuleIdFor<T>) -> DispatchResult {
		let capsule = Capsules::<T>::get(capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;

		ensure!(capsule.status == Status::FinalDeletion, Error::<T>::IncorrectCapsuleStatus);

		Capsules::<T>::remove(&capsule_id);
		CapsuleClearCursors::<T>::remove(&capsule_id);
		Self::deposit_event(Event::<T>::CapsuleDeleted { capsule_id });

		Ok(())
	}

	// Start destroying a capsule
}
