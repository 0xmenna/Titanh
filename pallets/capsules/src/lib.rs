// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

mod capsule;
mod container;
mod impl_utils;
mod impls;
mod types;
pub use types::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	use core::fmt::Debug;

	// Import various useful types required by all FRAME pallets.
	use super::*;
	use capsule::{CapsuleIdFor, *};
	use common_types::{Balance, CidFor, ContentSize, HashOf, Time};
	use container::*;
	use frame_support::{
		pallet_prelude::{StorageDoubleMap, ValueQuery, *},
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
		/// Type in which we record balances
		type Balance: Balance;
		/// Type for managing time
		type Timestamp: Time;
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
	}

	/// Capsules that wrap an IPFS CID
	#[pallet::storage]
	#[pallet::getter(fn capsules)]
	pub type Capsules<T> = StorageMap<_, Twox64Concat, CapsuleIdFor<T>, CapsuleMetadataOf<T>>;

	/// Capsule owners waiting for approval
	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub type OwnersWaitingApprovals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		HashOf<T>,
		Approval,
		ValueQuery,
	>;

	/// Followers of capsules
	#[pallet::storage]
	#[pallet::getter(fn followers)]
	pub type CapsuleFollowers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Twox64Concat,
		CapsuleIdFor<T>,
		Follower,
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
			cid: CidFor<T>,
			/// Size in bytes of the underline content
			size: ContentSize,
			/// App specific metadata
			app_data: Vec<u8>,
		},
		/// A waiting approval has been approved
		CapsuleOwnershipApproved {
			// Capsule identifier
			id: CapsuleIdFor<T>,
			who: T::AccountId,
		},
		/// Shared capsule ownership
		CapsuleSharedOwnership { id: CapsuleIdFor<T>, who: T::AccountId },
	}

	/// Errors that can be returned by this pallet.
	///
	/// Errors tell users that something went wrong so it's important that their naming is
	/// informative. Similar to events, error documentation is added to a node's metadata so it's
	/// equally important that they have helpful documentation associated with them.
	///
	/// This type of runtime error can be up to 4 bytes in size should you want to return additional
	/// information.
	#[pallet::error]
	pub enum Error<T> {
		/// Account has not app specific permissions
		AppPermissionDenied,
		/// Invalid owners
		TooManyOwners,
		/// Invalid App specific metadata
		BadAppData,
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Upload capsule dispatchable function
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn upload_capsule(
			origin: OriginFor<T>,
			app: AppIdFor<T>,
			owner: Option<T::AccountId>,
			capsule: CapsuleUploadData<CidFor<T>, BlockNumberFor<T>>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			ensure!(
				T::Permissions::has_account_permissions(&who, app.clone()),
				Error::<T>::AppPermissionDenied
			);
			// If no owner is provided as input, then the signer automatically becomes the owner.
			// Otherwise the ownership is passed to the input account
			let ownership = owner
				.map(|owner| Ownership::Other(owner))
				.unwrap_or_else(|| Ownership::Signer(who));
			// capsule id = hash(app + encoded_metadata)
			let capsule_id =
				Self::compute_capsule_id(app.clone(), capsule.encoded_metadata.clone());

			Self::upload_capsule_from(capsule_id, app, ownership, capsule)
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

			let capsule = Capsules::<T>::get(&capsule_id);
			if let Some(mut capsule) = capsule {
				// Try to approve a capsule waiting approval, if any
				Self::try_approve_capsule_ownership(&who, &capsule_id)?;
				// Try to add the owner to capsule owners, if it does not exceeds the vector bounds
				Self::try_add_owner(&who, &mut capsule.owners)?;

				// Emit Event
				Self::deposit_event(Event::<T>::CapsuleOwnershipApproved { id: capsule_id, who });

				Ok(())
			} else {
				Err(Error::<T>::InvalidCapsuleId.into())
			}
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

			// Obtain the capsule from the owner `who`
			// Dispatches an error if `who` is not an owner of the capsule
			let capsule = Self::capsule_from_owner(&who, &capsule_id)?;
			// check that `other_owner` is not already an owner
			ensure!(capsule.owners.binary_search(&other_owner).is_err(), Error::<T>::AlreadyOwner);
			// Add a waiting approval, only if there is not already the same one
			ensure!(
				OwnersWaitingApprovals::<T>::get(&other_owner, &capsule_id) == Approval::None,
				Error::<T>::AccountAlreadyInWaitingApprovals
			);
			OwnersWaitingApprovals::<T>::insert(&who, &capsule_id, Approval::None);

			// Emit Event
			Self::deposit_event(Event::<T>::CapsuleSharedOwnership { id: capsule_id, who });

			Ok(())
		}
	}
}
