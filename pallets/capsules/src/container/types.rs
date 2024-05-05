use crate::{AppData, AppIdFor, Config};
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::*;
use frame_system::Config as SystemConfig;
use scale_info::TypeInfo;
use sp_core::Get;

/// Details of a container (a collection of capsules)
pub type ContainerDetailsOf<T> = ContainerMetadata<
	<T as SystemConfig>::AccountId,
	<T as Config>::MaxOwners,
	<T as Config>::MaxEncodedAppMetadata,
	AppIdFor<T>,
>;

/// Container identifier
pub type ContainerIdOf<T> = HashOf<T>;

/// key in a container
pub type KeyOf<T> = BoundedString<<T as Config>::StringLimit>;

#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(MaxAccounts, S))]
pub struct ContainerMetadata<AccountId, MaxAccounts, S, AppId>
where
	MaxAccounts: Get<u32>,
	S: Get<u32>,
{
	/// Status of the container.
	/// Indicates who can attach and detach capsules to/from a container
	pub status: ContainerStatus,
	/// The number of keys in the container
	pub size: u32,
	/// The owners of the container.
	/// Each owner of the container is also owner of the capsules associated to the keys
	pub owners: Accounts<AccountId, MaxAccounts>,
	/// App specific metadata
	pub app_data: AppData<AppId, S>,
}

impl<AccountId, MaxAccounts, S, AppId> ContainerMetadata<AccountId, MaxAccounts, S, AppId>
where
	MaxAccounts: Get<u32>,
	S: Get<u32>,
{
	pub fn set_status(&mut self, status: ContainerStatus) {
		self.status = status;
	}
}

#[derive(Encode, Decode, MaxEncodedLen, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum ContainerStatus {
	#[default]
	RequiresOwnership,
	Public,
}
