use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::{
	Accounts, BlockNumberFor, BoundedString, CidFor, ContentSize, EncodedData, HashOf,
};
use frame_system::Config as SystemConfig;
use scale_info::TypeInfo;
use sp_core::Get;
use sp_std::prelude::*;

/// An application specific identifier
pub type AppId = u32;

/// Capsule identifier
pub type CapsuleIdFor<T> = HashOf<T>;

/// Account balance
pub type BalanceOf<T> = <T as Config>::Balance;

// Metadata associated to capsules
pub type CapsuleMetadataOf<T> = CapsuleMetadata<
	CidFor<T>,
	BlockNumberFor<T>,
	<T as SystemConfig>::AccountId,
	<T as Config>::MaxOwners,
	<T as Config>::MaxEncodedAppMetadata,
>;

/// Details of a document (a collection of capsules)
pub type DocumentDetailsOf<T> = DocumentMetadata<
	<T as SystemConfig>::AccountId,
	<T as Config>::MaxOwners,
	<T as Config>::MaxEncodedAppMetadata,
>;

// Actual type of a capsule metadata
#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(MaxAccounts, S))]
pub struct CapsuleMetadata<Cid, BlockNumber, AccountId, MaxAccounts, S>
where
	MaxAccounts: Get<u32>,
	S: Get<u32>,
{
	/// IPFS cid that points to the content
	pub cid: Cid,
	/// Size in bytes of the underline content
	pub size: ContentSize,
	/// The block number at which pinning nodes will stop pinning
	pub ending_retention_block: BlockNumber,
	/// The account owners of the capsule
	pub owners: Accounts<AccountId, MaxAccounts>,
	/// The types of followers allowed for the capsule
	pub followers_status: FollowersStatus,
	/// App specific metadata
	pub app_data: AppData<S>,
}

#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(S))]
pub struct AppData<S: Get<u32>> {
	/// An application specific identifier
	pub app_id: AppId,
	/// The app scale encoded data
	pub data: EncodedData<S>,
}

/// The type of capsule follower
#[derive(Encode, Decode, MaxEncodedLen, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub enum Follower {
	#[default]
	Basic,
	Privileged,
}

/// What kind of followers are allowed for a given capsule
#[derive(Encode, Decode, MaxEncodedLen, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum FollowersStatus {
	#[default]
	None,
	Basic,
	Privileged,
	All,
}

/// Document identifier
pub type DocumentIdOf<T> = HashOf<T>;

/// key of an underlining document
pub type KeyOf<T> = BoundedString<<T as Config>::StringLimit>;

#[derive(Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(MaxAccounts, S))]
pub struct DocumentMetadata<AccountId, MaxAccounts, S>
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
	pub app_data: AppData<S>,
}
