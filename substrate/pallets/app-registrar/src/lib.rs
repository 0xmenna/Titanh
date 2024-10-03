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
        pallet_prelude::{ValueQuery, *},
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, Saturating},
        FixedPointOperand,
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
    #[pallet::getter(fn app_permissions)]
    pub type AppPermissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AppId,
        Blake2_128Concat,
        T::AccountId,
        PermissionState,
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
        AppCreated {
            /// The account that created the app.
            who: T::AccountId,
            /// The account who set the new value.
            app_id: T::AppId,
        },
        SettedSubscriptionStatus {
            app_id: T::AppId,
            status: AppSubscriptionStatus,
        },
        AccountPermissionWaitingApproval {
            app_id: T::AppId,
            who: T::AccountId,
        },
        NewAccountPermission {
            app_id: T::AppId,
            who: T::AccountId,
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
        InvalidAppId,
        NotOwner,
        IncorrectStatus,
        NotAllowed,
        BadPermissions,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Creates an app instance and the caller becomes the owner.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn create_app(origin: OriginFor<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;
            // Increment the app id by one
            let mut index = CurrentAppId::<T>::get();
            index.saturating_inc();
            CurrentAppId::<T>::put(index);

            AppMetadata::<T>::insert(
                index,
                AppDetails {
                    owner: who.clone(),
                    status: Default::default(),
                },
            );

            AppPermissions::<T>::insert(index, who.clone(), PermissionState::Active);

            Self::deposit_event(Event::<T>::AppCreated { who, app_id: index });
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

            AppMetadata::<T>::try_mutate(app_id, |maybe_app_metadata| {
                if let Some(app_metadata) = maybe_app_metadata {
                    ensure!(who == app_metadata.owner, Error::<T>::NotOwner);

                    ensure!(
                        app_metadata.status != subscription_status,
                        Error::<T>::IncorrectStatus
                    );

                    app_metadata.status = subscription_status.clone();

                    Self::deposit_event(Event::<T>::SettedSubscriptionStatus {
                        app_id,
                        status: subscription_status,
                    });
                    Ok(())
                } else {
                    Err(Error::<T>::InvalidAppId.into())
                }
                // Return a successful `DispatchResult`
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn enable_account_permissions(
            origin: OriginFor<T>,
            app_id: T::AppId,
            permissions_receiver: T::AccountId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            let app_metadata = AppMetadata::<T>::get(app_id).ok_or(Error::<T>::InvalidAppId)?;
            ensure!(who == app_metadata.owner, Error::<T>::NotOwner);

            ensure!(
                AppSubscriptionStatus::SelectedByOwner == app_metadata.status,
                Error::<T>::IncorrectStatus
            );
            ensure!(
                AppPermissions::<T>::get(app_id, &permissions_receiver).is_none(),
                Error::<T>::BadPermissions
            );
            AppPermissions::<T>::insert(
                app_id,
                &permissions_receiver,
                PermissionState::WaitingApproval,
            );

            Self::deposit_event(Event::<T>::AccountPermissionWaitingApproval {
                app_id,
                who: permissions_receiver,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn subscribe_to_app(origin: OriginFor<T>, app_id: T::AppId) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;
            // Check that the status is set to Anyone.
            ensure!(
                AppSubscriptionStatus::Anyone
                    == AppMetadata::<T>::get(app_id)
                        .ok_or(Error::<T>::InvalidAppId)?
                        .status,
                Error::<T>::IncorrectStatus
            );
            ensure!(
                AppPermissions::<T>::get(app_id, &who).is_none(),
                Error::<T>::BadPermissions
            );
            AppPermissions::<T>::insert(app_id, &who, PermissionState::Active);

            Self::deposit_event(Event::<T>::NewAccountPermission { app_id, who });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn approve_app_permission(origin: OriginFor<T>, app_id: T::AppId) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;
            // Check that the status is set to Anyone.

            AppPermissions::<T>::get(app_id, &who)
                .map(|permissions_state| {
                    assert!(
                        permissions_state == PermissionState::WaitingApproval,
                        "Account is in an already active state"
                    )
                })
                .ok_or(Error::<T>::BadPermissions)?;

            AppPermissions::<T>::insert(app_id, &who, PermissionState::Active);
            Self::deposit_event(Event::<T>::NewAccountPermission { app_id, who });
            Ok(())
        }
    }
}
