use crate::{AppData, AppIdFor, Config, FollowersStatus};
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::*;
use frame_system::Config as SystemConfig;
use scale_info::TypeInfo;
use sp_core::Get;

/// Details of a document (a collection of capsules)
pub type DocumentDetailsOf<T> = DocumentMetadata<
	<T as SystemConfig>::AccountId,
	<T as Config>::MaxOwners,
	<T as Config>::MaxEncodedAppMetadata,
	AppIdFor<T>,
>;

/// Document identifier
pub type DocumentIdOf<T> = HashOf<T>;

/// key of an underlining document
pub type KeyOf<T> = BoundedString<<T as Config>::StringLimit>;

#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(MaxAccounts, S))]
pub struct DocumentMetadata<AccountId, MaxAccounts, S, AppId>
where
	MaxAccounts: Get<u32>,
	S: Get<u32>,
{
	/// The number of keys in the document
	pub size: u32,
	/// The owners of the document.
	/// Each owner of the document is also owner of the capsules associated to the keys
	pub owners: Accounts<AccountId, MaxAccounts>,
	/// The types of followers allowed for the capsule.
	/// Each follower will also be a follower of the underline capsules.
	pub followers_status: FollowersStatus,
	/// App specific metadata
	pub app_data: AppData<AppId, S>,
}
