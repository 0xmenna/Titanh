// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.

mod capsule;
mod container;
mod impl_utils;
mod types;

pub use pallet::*;

pub use capsule::*;
pub use container::*;
use sp_std::vec::Vec;
pub use types::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	use core::fmt::Debug;

	// Import various useful types required by all FRAME pallets.
	use super::*;
	use capsule::CapsuleIdFor;
	use common_types::{ContentSize, HashOf};
	use container::ContainerIdOf;
	use frame_support::{
		pallet_prelude::{StorageDoubleMap, *},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	use pallet_app_registrar::PermissionsApp;

	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A static prefix to compute a capsule id
		#[pallet::constant]
		type CapsuleIdPrefix: Get<&'static [u8]>;
		/// A static prefix to compute a container id
		#[pallet::constant]
		type ContainerIdPrefix: Get<&'static [u8]>;
		/// The maximum size of the encoded app specific metadata
		#[pallet::constant]
		type MaxEncodedAppMetadata: Get<u32> + Debug + Clone;
		/// The maximum number of owners per capsule/document
		#[pallet::constant]
		type MaxOwners: Get<u32> + Debug + Clone;
		/// The maximum length of a capsule key in a container stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32> + Clone;
		/// Permissions for accounts to perform operations under some application
		type Permissions: PermissionsApp<Self::AccountId>;
		/// Max number of items to destroy per `destroy_capsule_ownership_approvals`, `destroy_followers` and `destroy_container_keys` call.
		///
		/// Must be configured to result in a weight that makes each call fit in a block.
		#[pallet::constant]
		type RemoveItemsLimit: Get<u32>;
		/// Minimum number of blocks for a capsule retention period
		#[pallet::constant]
		type MinimumRetentionPeriod: Get<u32>;
		/// The IPFS CID length
		#[pallet::constant]
		type CidLength: Get<u32> + Clone;
	}

	/// Capsules that wrap an IPFS CID
	#[pallet::storage]
	#[pallet::getter(fn capsules)]
	pub type Capsules<T> = StorageMap<_, Twox64Concat, CapsuleIdFor<T>, CapsuleMetadataOf<T>>;

	/// Capsule owners waiting for approval
	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub type OwnersWaitingApprovals<T: Config> =
		StorageDoubleMap<_, Twox64Concat, HashOf<T>, Blake2_128Concat, T::AccountId, Approval>;

	/// Followers of capsules
	#[pallet::storage]
	#[pallet::getter(fn followers)]
	pub type CapsuleFollowers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CapsuleIdFor<T>,
		Twox64Concat,
		T::AccountId,
		Follower,
	>;

	/// Containers in which a capsule is defined, giving its associated key
	// This is needed for efficiency reasons.
	// If a capsule is being deleted, to avoid an undefined number of transactions for the deletion,
	// we define the storage to know in what containers a capsule is defined and with what key.
	#[pallet::storage]
	pub type CapsuleContainers<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		CapsuleIdFor<T>,
		Twox64Concat,
		ContainerIdOf<T>,
		KeyOf<T>,
	>;

	/// Container with different capsules identified by a key
	#[pallet::storage]
	#[pallet::getter(fn container_get)]
	pub type Container<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ContainerIdOf<T>,
		Blake2_128Concat,
		KeyOf<T>,
		CapsuleIdFor<T>,
	>;

	/// Details of a container
	#[pallet::storage]
	#[pallet::getter(fn container_details)]
	pub type ContainerDetails<T: Config> =
		StorageMap<_, Twox64Concat, ContainerIdOf<T>, ContainerDetailsOf<T>>;

	/// Events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has successfully set a new value.
		CapsuleUploaded {
			/// Capsule identifier
			id: CapsuleIdFor<T>,
			/// Application identifer
			app_id: AppIdFor<T>,
			/// IPFS cid that points to the content
			cid: Vec<u8>,
			/// Size in bytes of the underline content
			size: ContentSize,
			/// App specific metadata
			app_data: Vec<u8>,
			/// Owner
			ownership: Ownership<T::AccountId>,
			/// Followers status
			followers_status: FollowersStatus,
		},
		/// A waiting approval has been approved
		OwnershipApproved {
			// Capsule identifier
			id: HashOf<T>,
			// Approval account
			who: T::AccountId,
			// Type of approval (capsule or container)
			approval: Approval,
		},
		/// Shared capsule ownership
		SharedOwnership {
			id: HashOf<T>,
			who: T::AccountId,
			waiting_approval: Approval,
		},
		/// Capsule Followers Status changed
		CapsuleFollowersStatusChanged {
			capsule_id: CapsuleIdFor<T>,
			status: FollowersStatus,
		},
		/// A capsule has been followed
		CapsuleFollowed {
			capsule_id: CapsuleIdFor<T>,
			follower: T::AccountId,
		},
		/// The content pointed by a capsule has changed
		CapsuleContentChanged {
			capsule_id: CapsuleIdFor<T>,
			old_cid: Vec<u8>,
			cid: Vec<u8>,
			size: ContentSize,
		},
		/// The endind retention block has been extended
		CapsuleEndingRetentionBlockExtended {
			capsule_id: CapsuleIdFor<T>,
			at_block: BlockNumberFor<T>,
		},
		/// A priviledged follower is added to a waiting for approval state
		PrivilegedFollowerWaitingToApprove {
			capsule_id: CapsuleIdFor<T>,
			who: T::AccountId,
		},
		/// A waiting approval has been approved
		NewPrivilegedFollower {
			capsule_id: CapsuleIdFor<T>,
			who: T::AccountId,
		},
		/// Started the deletion of a capsule
		CapsuleStartedDestroying {
			capsule_id: CapsuleIdFor<T>,
		},
		/// Capsule items have been deleted
		CapsuleItemsDeleted {
			capsule_id: CapsuleIdFor<T>,
			/// Wether all items have been deleted
			removal_completion: bool,
			// type of items deleted
			items: CapsuleItems,
		},
		/// Capsule containers deleted
		CapsuleContainersDeleted {
			capsule_id: CapsuleIdFor<T>,
			/// Wether all itemss have been deleted
			removal_completion: bool,
		},
		/// Capsule deleted
		CapsuleDeleted {
			capsule_id: CapsuleIdFor<T>,
			cid: Vec<u8>,
		},
		/// Container uploaded
		ContainerCreated {
			container_id: ContainerIdOf<T>,
			app_id: AppIdFor<T>,
			app_data: Vec<u8>,
			ownership: Ownership<T::AccountId>,
		},
		CapsuleAttached {
			container_id: ContainerIdOf<T>,
			key: Vec<u8>,
			capsule_id: CapsuleIdFor<T>,
		},
		CapsuleDetached {
			container_id: ContainerIdOf<T>,
			key: Vec<u8>,
			capsule_id: CapsuleIdFor<T>,
		},
		ContainerStatusChanged {
			container_id: ContainerIdOf<T>,
			status: ContainerStatus,
		},
	}

	/// Errors that can be returned by this pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// Account has not app specific permissions
		AppPermissionDenied,
		/// Invalid owners
		TooManyOwners,
		/// Invalid App specific metadata
		BadAppData,
		/// Invalid cid
		BadCid,
		/// Capsule with that id already exists
		CapsuleIdAlreadyExists,
		/// Account has no waiting approvals
		NoWaitingApproval,
		/// Capsule does not exits
		InvalidCapsuleId,
		/// Account is not an owner
		BadOriginForOwnership,
		/// The account is already an owner
		AlreadyOwner,
		/// Account already waiting for approval
		AccountAlreadyInWaitingApprovals,
		/// Invalid followers status
		BadFollowersStatus,
		/// An account is already a follower
		AlreadyFollower,
		/// Invalid block number for a retention extension
		BadBlockNumber,
		/// Invalid deletion stage
		IncorrectCapsuleStatus,
		/// Invalid Container
		InvalidContainerId,
		/// Invalid key format of a container
		BadKeyFormat,
		/// Invalid key
		BadKey,
		/// Invalid number of blocks for the retention period
		BadRetentionPeriod,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/*
		Capsule related dispatchables
		*/

		/// Upload capsule logic
		///
		/// Vulnerability NOTE:
		/// In the current implementation an account could update a capsule by specifying the `size` parameter,
		/// in the capsule metadata, that is not consistent within the actual content stored on IPFS.
		/// The reason of such parameter to exist is to allow, in future implementations, a renting mechanism.
		/// In fact, a fee can be charged to the uploader, based on the size and rentention time.
		///
		/// To solve such vulnerability, pinning nodes should verify the validity of the content size, and sign a message that can be validated on chain.
		/// This can be implemented in future versions.
		///
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn upload_capsule(
			origin: OriginFor<T>,
			app: AppIdFor<T>,
			other_owner: Option<T::AccountId>,
			capsule: CapsuleUploadData<BlockNumberFor<T>>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::upload_capsule_from(who, app, other_owner, capsule)
		}

		/// Approves an ownership request for a given capsule
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn approve_capsule_ownership(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::approve_capsule_ownership_from(who, capsule_id)
		}

		/// Share the ownership of a capsule with another account
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn share_capsule_ownership(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
			other_owner: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::share_capsule_ownership_from(who, capsule_id, other_owner)
		}

		/// Set Follower status of a capsule
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn set_capsule_followers_status(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
			followers_status: FollowersStatus,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::set_capsule_followers_status_from(who, capsule_id, followers_status)
		}

		/// Follow a capsule
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn follow_capsule(origin: OriginFor<T>, capsule_id: CapsuleIdFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::follow_capsule_from(who, capsule_id)
		}

		/// Updates the content of a capsule.
		/// By means of changing the IPFS CID and size (see vulnerability in the upload extrinisc).
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn update_capsule_content(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
			cid: Vec<u8>,
			size: ContentSize,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::update_capsule_content_from(who, capsule_id, cid, size)
		}

		/// Extends the ending retention block of a capsule
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn extend_ending_retention_block(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
			at_block: BlockNumberFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::extend_ending_retention_block_from(who, capsule_id, at_block)
		}

		/// Adds priviledged followers, by adding it to a waiting approval state
		/// In order to become a priviledged follower the target account must agree
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn add_priviledged_follower(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
			follower: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::add_priviledged_follower_from(who, capsule_id, follower)
		}

		/// Approves a privileged follower request
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn approve_privileged_follow(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::approve_privileged_follow_from(who, capsule_id)
		}

		/// Start the deletion of a capsule
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn start_destroy_capsule(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::start_destroy_capsule_from(who, capsule_id)
		}

		/// Deletes all ownership approvals of a capsule, up to `T::RemoveItemsLimit`
		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn destroy_capsule_ownership_approvals(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_signed(origin)?;
			Self::destroy_ownership_approvals_from(capsule_id, T::RemoveItemsLimit::get())
		}

		/// Deletes all followers of a capsule, up to `T::RemoveItemsLimit`
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn destroy_capsule_followers(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_signed(origin)?;
			Self::destroy_followers_from(capsule_id, T::RemoveItemsLimit::get())
		}

		/// Deletes all capsules within a container and the entries of `CapsuleContainers up to `T::RemoveItemsLimit`
		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn destroy_capsule_container_keys(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_signed(origin)?;
			Self::destroy_container_keys_of(capsule_id, T::RemoveItemsLimit::get())
		}

		/// Finish the destroy of a capsule
		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn finish_destroy_capsule(
			origin: OriginFor<T>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_signed(origin)?;
			Self::finish_destroy_capsule_from(capsule_id)
		}

		/*
		Container related dispatchables
		*/

		/// Create a container
		#[pallet::call_index(14)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn create_container(
			origin: OriginFor<T>,
			app_id: AppIdFor<T>,
			maybe_other_owner: Option<T::AccountId>,
			container_metadata: Vec<u8>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::create_container_from(who, app_id, maybe_other_owner, container_metadata)
		}

		/// Approves an ownership request for a given container
		#[pallet::call_index(15)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn approve_container_ownership(
			origin: OriginFor<T>,
			container_id: ContainerIdOf<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::approve_container_ownership_from(who, container_id)
		}

		/// Share the ownership of a container with another account
		#[pallet::call_index(16)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn share_container_ownership(
			origin: OriginFor<T>,
			container_id: ContainerIdOf<T>,
			other_owner: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::share_container_ownership_from(who, container_id, other_owner)
		}

		/// Puts a capsule into a container
		#[pallet::call_index(17)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn container_put(
			origin: OriginFor<T>,
			container_id: ContainerIdOf<T>,
			key: Vec<u8>,
			capsule_id: CapsuleIdFor<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::attach_capsule_to_container_from(who, container_id, key, capsule_id)
		}

		/// Removes a capsule from a container
		#[pallet::call_index(18)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn container_remove(
			origin: OriginFor<T>,
			container_id: ContainerIdOf<T>,
			key: Vec<u8>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::detach_capsule_from_container(who, container_id, key)
		}

		/// Change Container Status
		#[pallet::call_index(19)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn change_container_status(
			origin: OriginFor<T>,
			container_id: ContainerIdOf<T>,
			container_status: ContainerStatus,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			Self::change_container_status_from(who, container_id, container_status)
		}
	}
}
