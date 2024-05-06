use crate::{AppData, AppIdFor, Config, DeletionCompletion, FollowersStatus};
use codec::{Decode, Encode, MaxEncodedLen};
use common_types::*;
use frame_system::Config as SystemConfig;
use scale_info::TypeInfo;
use sp_core::{Get, RuntimeDebug};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

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
	// Capsule status
	pub status: Status,
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

impl<Cid, BlockNumber, AccountId, MaxAccounts, S, AppId>
	CapsuleMetadata<Cid, BlockNumber, AccountId, MaxAccounts, S, AppId>
where
	MaxAccounts: Get<u32>,
	S: Get<u32>,
{
	pub fn set_status(&mut self, status: Status) {
		self.status = status;
	}

	pub fn set_followers_status(&mut self, followers_status: FollowersStatus) {
		self.followers_status = followers_status;
	}
}

#[derive(Encode, Decode, Clone, Eq, Default, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Status {
	#[default]
	Live,
	ItemsDeletion(DeletionCompletion),
	FinalDeletion,
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

pub struct CapsuleMetaBuilder<T: Config> {
	app_id: AppIdFor<T>,
	owners: Vec<T::AccountId>,
	upload_data: CapsuleUploadData<CidFor<T>, BlockNumberFor<T>>,
}

impl<T: Config> CapsuleMetaBuilder<T> {
	pub fn new(
		app_id: AppIdFor<T>,
		owners: Vec<T::AccountId>,
		upload_data: CapsuleUploadData<CidFor<T>, BlockNumberFor<T>>,
	) -> Self {
		Self { app_id, owners, upload_data }
	}

	pub fn build(self) -> Result<CapsuleMetadataOf<T>, DispatchError> {
		Ok(CapsuleMetadata {
			status: Default::default(),
			cid: self.upload_data.cid,
			size: self.upload_data.size,
			ending_retention_block: self.upload_data.ending_retention_block,
			owners: self.owners.try_into().map_err(|_| crate::Error::<T>::TooManyOwners)?,
			followers_status: self.upload_data.followers_status,
			app_data: AppData {
				app_id: self.app_id,
				data: EncodedData::from_slice(&self.upload_data.encoded_metadata)
					.map_err(|_| crate::Error::<T>::BadAppData)?,
			},
		})
	}
}
