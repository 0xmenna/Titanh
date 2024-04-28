use super::{CapsuleIdFor, CapsuleMetaBuilder, CapsuleUploadData};
use crate::{
	capsule::Status, AppIdFor, Approval, CapsuleFollowers, Capsules, Config, Error, Event,
	Follower, FollowersStatus, IdComputation, OwnersWaitingApprovals, Ownership, Pallet,
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
		Self::try_approve_capsule_ownership(&who, &capsule_id)?;
		// Try to add the owner to capsule owners, if it does not exceeds the vector bounds
		Self::try_add_owner(&who, &mut capsule.owners)?;

		// Emit Event
		Self::deposit_event(Event::<T>::CapsuleOwnershipApproved { id: capsule_id, who });

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
		// check that `other_owner` is not already an owner
		ensure!(capsule.owners.binary_search(&other_owner).is_err(), Error::<T>::AlreadyOwner);

		// Add a waiting approval, only if there is not already the same one
		if let Some(waiting_approval) = OwnersWaitingApprovals::<T>::get(&capsule_id, &other_owner)
		{
			debug_assert!(waiting_approval == Approval::Capsule, "The existant approval must be associated to a capsule because identifiers are based on their own prefixes");
			return Err(Error::<T>::AccountAlreadyInWaitingApprovals.into());
		}
		OwnersWaitingApprovals::<T>::insert(&capsule_id, &who, Approval::Capsule);

		// Emit Event
		Self::deposit_event(Event::<T>::CapsuleSharedOwnership { id: capsule_id, who });

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
			CapsuleFollowers::<T>::get(&who, &capsule_id).is_none(),
			Error::<T>::AlreadyFollower
		);
		CapsuleFollowers::<T>::insert(&who, &capsule_id, Follower::Basic);

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
			CapsuleFollowers::<T>::get(&follower, &capsule_id).unwrap_or_default()
				== Follower::Basic,
			Error::<T>::AlreadyFollower
		);
		CapsuleFollowers::<T>::insert(
			&follower,
			&capsule_id,
			Follower::WaitingApprovalForPrivileged,
		);

		// Emit event
		Self::deposit_event(Event::<T>::PrivilegedFollowerWaitingToApprove {
			capsule_id,
			who: follower,
		});

		Ok(())
	}

	pub fn aprove_privileged_follow_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
	) -> DispatchResult {
		if let Some(capsule) = Capsules::<T>::get(&capsule_id) {
			Self::ensure_capsule_liveness(&capsule)?;
			// check that `who` is in a waiting approval state
			ensure!(
				CapsuleFollowers::<T>::get(&who, &capsule_id).unwrap_or_default()
					== Follower::WaitingApprovalForPrivileged,
				Error::<T>::NoWaitingApproval
			);
			CapsuleFollowers::<T>::insert(&who, &capsule_id, Follower::Privileged);

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

		let owners = match ownership {
			Ownership::Signer(who) => {
				// Set the signer as the owner
				vec![who]
			},
			Ownership::Other(who) => {
				// Adding a waiting approval for the capsule
				// The owner must accept it before becoming an owner
				OwnersWaitingApprovals::<T>::insert(capsule_id.clone(), who, Approval::Capsule);
				Vec::new()
			},
		};

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
		});

		Ok(())
	}

	pub fn start_destroy_capsule_from(
		who: T::AccountId,
		capsule_id: CapsuleIdFor<T>,
	) -> DispatchResult {
		let mut capsule = Capsules::<T>::get(&capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;
		assert!(
			capsule.status != Status::Destroying,
			"The capsule is already in the destroying state"
		);
		// If the retention period has elapsed, anyone is allowed to destroy the capsule.
		// This is to increase the level of decentralization.
		// Else, only an owner is capable to start the deletion of a capsule
		if capsule.ending_retention_block > <frame_system::Pallet<T>>::block_number() {
			ensure!(capsule.owners.binary_search(&who).is_ok(), Error::<T>::BadOriginForOwnership);
		}
		capsule.status = Status::Destroying;

		Ok(())
	}

	pub fn destroy_ownership_approvals_from(capsule_id: CapsuleIdFor<T>) -> DispatchResult {
		let capsule = Capsules::<T>::get(capsule_id).ok_or(Error::<T>::InvalidCapsuleId)?;
		// Check if the removal of the capsule has been initiated
		ensure!(capsule.status == Status::Destroying, Error::<T>::LiveCapsule);
		for (i, who) in OwnersWaitingApprovals::<T>::iter_key_prefix(&capsule_id).enumerate() {}
		Ok(())
	}

	// Start destroying a capsule
}
