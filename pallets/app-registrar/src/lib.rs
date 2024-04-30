// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

mod traits;
mod types;
pub use traits::*;
pub use types::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::{
		pallet,
		pallet_prelude::{ValueQuery, *},
		Blake2_128Concat, Twox64Concat,
	};
	use frame_system::{pallet, pallet_prelude::*};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Saturating},
		AccountId32, DispatchError, FixedPointOperand,
	};

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
		/// Identifier for the class of application.
		type AppId: Member
			+ Parameter
			+ Clone
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ FixedPointOperand
			+ Default
			+ AtLeast32BitUnsigned;
	}

	#[pallet::storage]
	#[pallet::getter(fn app_id)]
	pub type CurrentAppId<T: Config> = StorageValue<_, T::AppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn app_permission)]
	pub type AppPermission<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AppId,
		Blake2_128Concat,
		T::AccountId,
		bool,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn app_metadata)]
	pub type AppMetadata<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AppId, AppDetails<T::AccountId>>;

	/// Events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]

	pub enum Event<T: Config> {
		/// A user has successfully set a new value.
		CreatedApp {
			/// The new value set.
			owner: T::AccountId,
			/// The account who set the new value.
			app_id: T::AppId,
		},
		SettedSubscriptionStatus {
			app_id: T::AppId,
			status: AppSubscriptionStatus,
		},
		AddedPermissionAccount {
			app_id: T::AppId,
			account_id:T::AccountId,
		},
		AddedAccount {
			account_id: T::AccountId,
			app_id: T::AppId,
		},
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
		/// The value retrieved was `None` as no value was previously set.
		AppNotExist,
		NotOwner,
		IncorrectStatus,
		NotAllowed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Upload capsule dispatchable function
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn create_app(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			// Increment the counter value by one
			let mut index = CurrentAppId::<T>::get();
			index.saturating_inc();
			// Update the storage AppOwners

			AppMetadata::<T>::insert(
				index,
				AppDetails { owner: who.clone(), status: Default::default() },
			);

			AppPermission::<T>::insert(index, who.clone(), true);

			Self::deposit_event(Event::<T>::CreatedApp{
				owner: who,
				app_id: index
			});
			// Return a successful `DispatchResult`
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn set_subscription_status(
			origin: OriginFor<T>,
			app_id: T::AppId,
			subscription_status: AppSubscriptionStatus,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			let mut app_metadata = AppMetadata::<T>::get(app_id).ok_or(Error::<T>::AppNotExist)?;

			ensure!(who == app_metadata.owner, Error::<T>::NotOwner);

			ensure!(app_metadata.status != subscription_status, Error::<T>::IncorrectStatus);

			app_metadata.status = subscription_status;

			Self::deposit_event(Event::<T>::SettedSubscriptionStatus{
				app_id: app_id,
				status: app_metadata.status,
			});
			
			// Return a successful `DispatchResult`
			Ok(())
		}
		// TODO: Aggiungi una lista di attesa, uno storage in cui aggiungi tutti quelli che vogliono iscriversi all'app
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn enable_account_permission(
			origin: OriginFor<T>,
			app_id: T::AppId,
			account_to_add: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			let app_metadata = AppMetadata::<T>::get(app_id).ok_or(Error::<T>::AppNotExist)?;

			ensure!(who == app_metadata.owner, Error::<T>::NotOwner);

			ensure!(
				AppSubscriptionStatus::SelectedByOwner == app_metadata.status,
				Error::<T>::IncorrectStatus
			);

			AppPermission::<T>::insert(app_id, &account_to_add, true);
			
			Self::deposit_event(Event::<T>::AddedPermissionAccount{
				app_id: app_id,
				account_id: account_to_add,
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn subscribe_to_app_permission(
			origin: OriginFor<T>,
			app_id: T::AppId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			// Check that the status is set to Anyone.
			ensure!(
				AppSubscriptionStatus::Anyone
					== AppMetadata::<T>::get(app_id).ok_or(Error::<T>::AppNotExist)?.status,
				Error::<T>::IncorrectStatus
			);
			// Insert the accountId into the storage AppPermission
			AppPermission::<T>::insert(app_id, &who, true);

			Self::deposit_event(Event::<T>::AddedAccount{
				account_id: who,
				app_id: app_id,
			});
			Ok(())
		}
	}
}
