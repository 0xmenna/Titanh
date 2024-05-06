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
	}

	/// The number of pinning nodes that will pin the content underneath an IPFS cid
	#[pallet::storage]
	#[pallet::getter(fn content_replication_factor)]
	pub type ContentReplicationFactor<T: Config> = StorageValue<_, ReplicationFactor>;

	/// Events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]

	pub enum Event<T: Config> {
		/// A user has successfully set a new value.
		ContentReplicationFactorSet { factor: ReplicationFactor },
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the content replication factor associated to IPFS cids
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn set_content_replication_factor(
			origin: OriginFor<T>,
			factor: ReplicationFactor,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_root(origin)?;

			ContentReplicationFactor::<T>::put(factor);
			Self::deposit_event(Event::<T>::ContentReplicationFactorSet { factor });
			Ok(())
		}
	}
}
