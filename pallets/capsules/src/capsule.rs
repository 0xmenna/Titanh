use crate::{AppData, AppIdFor, Config, FollowersStatus};
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::*;
use frame_system::Config as SystemConfig;
use scale_info::TypeInfo;
use sp_core::{Get, RuntimeDebug};

/// Capsule identifier
pub type CapsuleIdFor<T> = HashOf<T>;

// Metadata associated to capsules
pub type CapsuleMetadataOf<T> = CapsuleMetadata<
	CidFor<T>,
	BlockNumberFor<T>,
	<T as SystemConfig>::AccountId,
	<T as Config>::MaxOwners,
	<T as Config>::MaxEncodedAppMetadata,
	AppIdFor<T>,
>;

// Actual type of a capsule metadata
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxAccounts, S))]
pub struct CapsuleMetadata<Cid, BlockNumber, AccountId, MaxAccounts, S, AppId>
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
	pub app_data: AppData<AppId, S>,
}

/// Data to upload
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct CapsuleUploadData<Cid, BlockNumber> {
	/// IPFS cid that points to the content
	pub cid: Cid,
	/// Size in bytes of the underline content
	pub size: ContentSize,
	/// The block number at which pinning nodes will stop pinning
	pub ending_retention_block: BlockNumber,
	/// The types of followers allowed for the capsule
	pub followers_status: FollowersStatus,
	/// App encoded_metadata
	pub encoded_metadata: Vec<u8>,
}
