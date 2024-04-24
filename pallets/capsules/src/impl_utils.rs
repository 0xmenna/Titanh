use crate::{capsule::CapsuleIdFor, AppIdFor, Capsules, Config, Pallet};
use codec::Encode;
use common_types::CidFor;
use sp_core::Hasher;

impl<T: Config> Pallet<T> {
	pub(super) fn compute_capsule_id(app_id: AppIdFor<T>, metadata: Vec<u8>) -> CapsuleIdFor<T> {
		let mut data = Vec::new();
		data.push(app_id.encode());
		data.push(metadata);

		T::Hashing::hash(&data.concat()[..])
	}

	pub fn capsule_exists(capsule_id: &CapsuleIdFor<T>) -> bool {
		Capsules::<T>::get(capsule_id).is_some()
	}
}
