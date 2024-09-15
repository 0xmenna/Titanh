// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod types;

use codec::Encode;
use common_types::PinningNodeIdOf;
use sp_core::Hasher;
use sp_std::vec::Vec;
pub use types::*;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub mod ed25519 {
	mod app_ed25519 {
		use common_types::PINNING;
		use sp_application_crypto::{app_crypto, ed25519};
		app_crypto!(ed25519, PINNING);
	}

	sp_application_crypto::with_pair! {
		/// An IPFS keypair using ed25519 as its crypto.
		pub type AuthorityPair = app_ed25519::Pair;
	}

	/// An IPFS signature using ed25519 as its crypto.
	pub type AuthoritySignature = app_ed25519::Signature;

	/// An IPFS identifier using ed25519 as its crypto.
	pub type AuthorityId = app_ed25519::Public;
}

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::{
		pallet_prelude::{ValueQuery, *},
		traits::ValidatorRegistration,
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	use sp_application_crypto::RuntimeAppPublic;
	use sp_runtime::traits::Convert;

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
		/// The maximum numbers of pinning nodes
		#[pallet::constant]
		type MaxPinningNodes: Get<u32>;
		/// A stable ID for a validator.
		type ValidatorId: Member
			+ Parameter
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TryFrom<Self::AccountId>;
		/// Validators registrar
		type ValidatorRegistrar: ValidatorRegistration<Self::ValidatorId>;
		/// A conversion from account ID to validator ID.
		///
		/// Its cost must be at most one storage read.
		type ValidatorIdOf: Convert<Self::AccountId, Option<Self::ValidatorId>>;
		/// The identifier type for an IPFS node, aka its public key.
		type IPFSNodeId: Parameter
			+ RuntimeAppPublic
			+ Ord
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;
		/// The registration message a validator signs with the registring IPFS node pubkey
		#[pallet::constant]
		type RegistrationMessage: Get<&'static [u8]>;
	}

	/// The number of pinning nodes that will pin the content underneath an IPFS cid
	#[pallet::storage]
	#[pallet::getter(fn content_replication_factor)]
	pub type ContentReplicationFactor<T: Config> = StorageValue<_, ReplicationFactor, ValueQuery>;

	/// The number of pinning nodes per validator
	#[pallet::storage]
	#[pallet::getter(fn pinning_nodes)]
	pub type NumOfPinningNodes<T> = StorageValue<_, u32, ValueQuery>;

	/// Metadata of validators' pinning nodes
	#[pallet::storage]
	#[pallet::getter(fn validators_pinning_nodes)]
	#[pallet::unbounded]
	pub type ValidatorsPinningNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ValidatorId, PinningNodesKeysOf<T>, ValueQuery>;

	/// The ring of the pinning nodes, by means of a circular vector of identifiers (hashes)
	#[pallet::storage]
	#[pallet::getter(fn pinning_nodes_ring)]
	pub type PinningNodesRing<T: Config> = StorageValue<_, PinningRing<T>, ValueQuery>;
	
	/// Events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]

	pub enum Event<T: Config> {
		/// The replication factor has been set
		ContentReplicationFactorSet { factor: ReplicationFactor },
		/// The number of pinning nodes per validator has been set
		PinningNodesPerValidatorSet { pinning_nodes: u32 },
		/// A registration of a new pinning node took place
		NewPinningNodeRegistration {
			validator: T::ValidatorId,
			registration: Registration<T::IPFSNodeId>,
		},
	}

	/// Errors that can be returned by this pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// Validator is not registered
		ValidatorNotRegistered,
		/// There is no validator associated to the signer account
		NoAssociatedValidatorId,
		/// There are too many ipfs keys for one validator
		TooManyIPFSKeys,
		/// Wrong ipfs key index
		WrongIpfsKeyIndex,
		/// Pinning node already in the ring
		PinningNodeAlreadyInRing,
		/// Too many Pinning nodes in the ring
		RingOutOfBounds,
		// Invalid signature
		InvalidRegistrationSignature,
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
			// Check that the extrinsic was signed by sudo.
			ensure_root(origin)?;

			ContentReplicationFactor::<T>::put(factor);
			Self::deposit_event(Event::<T>::ContentReplicationFactorSet { factor });

			Ok(())
		}

		/// Set the number of pinning nodes associated to each validator
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn set_pinning_nodes_per_validator(
			origin: OriginFor<T>,
			pinning_nodes: u32,
		) -> DispatchResult {
			// Check that the extrinsic was signed by sudo.
			ensure_root(origin)?;

			NumOfPinningNodes::<T>::put(pinning_nodes);
			Self::deposit_event(Event::<T>::PinningNodesPerValidatorSet { pinning_nodes });

			Ok(())
		}

		/// Register a validator pinning node
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn register_pinning_node(
			origin: OriginFor<T>,
			// The registration message signed with the IPFS node key
			registration: RegistrationMessageOf<T>,
			// Eventually the position of the pinning node key to replace
			// if `None`, then it will be appended
			maybe_pinning_node_idx: Option<PinningNodeIndex>,
		) -> DispatchResult {
			// Check that the extrinsic was signed by a validator.
			let who = ensure_signed(origin)?;
			let validator =
				T::ValidatorIdOf::convert(who).ok_or(Error::<T>::NoAssociatedValidatorId)?;
			ensure!(
				T::ValidatorRegistrar::is_registered(&validator),
				Error::<T>::ValidatorNotRegistered
			);

			// Try to insert the ipfs key into the pinning nodes keys of the validator
			let r = ValidatorsPinningNodes::<T>::try_mutate(
				&validator,
				|keys| -> Result<Registration<T::IPFSNodeId>, DispatchError> {
					if let Some(idx) = maybe_pinning_node_idx {
						ensure!(keys.len() > idx as usize, Error::<T>::WrongIpfsKeyIndex);
						let old_key = keys[idx as usize].clone();
						keys[idx as usize] = registration.key.clone();
						Ok(Registration::Substitution(old_key))
					} else {
						ensure!(
							keys.len() < NumOfPinningNodes::<T>::get() as usize,
							Error::<T>::TooManyIPFSKeys
						);
						keys.push(registration.key.clone());
						Ok(Registration::Addition)
					}
				},
			)?;

			let mut pinning_ring = match r.clone() {
				Registration::Addition => PinningNodesRing::<T>::get(),
				Registration::Substitution(old_key) => {
					Self::remove_old_pinning_node(&old_key, &validator)
				},
			};

			// Insert the new pinning node in the ring.
			// The position is based on the computed id: hash(registration.key + validator)
			let pinning_id = Self::compute_pinning_node_id(&registration.key, &validator);
			let idx = pinning_ring
				.binary_search(&pinning_id)
				.err()
				.ok_or(Error::<T>::PinningNodeAlreadyInRing)?;

			pinning_ring
				.try_insert(idx, pinning_id.clone())
				.map_err(|_| Error::<T>::RingOutOfBounds)?;

			PinningNodesRing::<T>::put(pinning_ring);

			// We verify the signature at last because is the most computationally intensive part
			if !registration.key.verify(&T::RegistrationMessage::get(), &registration.signature) {
				return Err(Error::<T>::InvalidRegistrationSignature.into());
			}

			Self::deposit_event(Event::<T>::NewPinningNodeRegistration {
				validator,
				registration: r,
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn compute_pinning_node_id(
		ipfs_key: &T::IPFSNodeId,
		validator_key: &T::ValidatorId,
	) -> PinningNodeIdOf<T> {
		let mut id = Vec::new();
		id.extend_from_slice(&ipfs_key.encode());
		id.extend_from_slice(&validator_key.encode());

		T::Hashing::hash(&id[..])
	}

	/// Removes pinning node identified by `old_key` and `validator` from the ring.
	/// Panics if is not in the ring
	fn remove_old_pinning_node(
		old_key: &T::IPFSNodeId,
		validator: &T::ValidatorId,
	) -> PinningRing<T> {
		let mut pinning_ring = PinningNodesRing::<T>::get();
		// The actual identifier of the old node in the ring
		let old_id = Self::compute_pinning_node_id(old_key, validator);

		let idx = pinning_ring.binary_search(&old_id).expect("Pinning node has to be in the ring");
		pinning_ring.remove(idx);

		pinning_ring
	}
}
