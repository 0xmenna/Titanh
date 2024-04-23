use codec::Encode;
use common_types::CidFor;
use sp_core::Hasher;

use crate::{capsule::CapsuleIdFor, AppIdFor, Config, Pallet};

impl<T: Config> Pallet<T> {
	pub(super) fn compute_capsule_id(app_id: AppIdFor<T>, metadata: Vec<u8>) -> CapsuleIdFor<T> {
		let mut data = Vec::new();
		data.push(app_id.encode());
		data.push(metadata);

		T::Hashing::hash(&data.concat()[..])
	}
}
