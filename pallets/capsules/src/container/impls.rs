use sp_runtime::DispatchResult;

use crate::{AppIdFor, Config, FollowersStatus, Pallet};

/// Container related logic
impl<T: Config> Pallet<T> {
	pub fn create_container_from(
		who: T::AccountId,
		app_id: AppIdFor<T>,
		maybe_other_owner: Option<T::AccountId>,
		followers_status: FollowersStatus,
		app_data: Vec<u8>,
	) -> DispatchResult {
		todo!()
	}
}
