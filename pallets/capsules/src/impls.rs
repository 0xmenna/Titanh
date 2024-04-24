use crate::{
	capsule::{CapsuleIdFor, CapsuleMetaBuilder, CapsuleUploadData},
	AppIdFor, Approvals, Capsules, Config, Error, Event, OwnersApprovals, Ownership, Pallet,
};
use common_types::{BlockNumberFor, CidFor};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
	pub fn upload_capsule_from(
		capsule_id: CapsuleIdFor<T>,
		app_id: AppIdFor<T>,
		ownership: Ownership<T::AccountId>,
		metadata: CapsuleUploadData<CidFor<T>, BlockNumberFor<T>>,
	) -> DispatchResult {
		ensure!(!Self::capsule_exists(&capsule_id), Error::<T>::InvalidCapsuleId);

		let owners = match ownership {
			Ownership::Signer(who) => {
				// Set the signer as the owner
				vec![who]
			},
			Ownership::Other(who) => {
				// Adding a waiting approval for the capsule
				// The owner must accept it before becoming an owner
				OwnersApprovals::<T>::insert(who, capsule_id.clone(), Approvals::Waiting);
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
}
