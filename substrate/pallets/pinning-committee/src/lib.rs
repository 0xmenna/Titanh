// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod types;

use codec::Encode;
use common_types::PinningNodeIdOf;
use frame_support::{
	ensure,
	pallet_prelude::{ValueQuery, *},
	traits::ValidatorRegistration,
	Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use sp_application_crypto::RuntimeAppPublic;
use sp_core::Hasher;
use sp_runtime::traits::Convert;
use sp_std::vec::Vec;

pub use pallet::*;
pub use types::*;

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

	/// The number of ipfs nodes per pinning node
	#[pallet::storage]
	#[pallet::getter(fn ipfs_replication_factor)]
	pub type NumOfIpfsReplicas<T> = StorageValue<_, u32, ValueQuery>;

	/// Ipfs keys of a validator node that still have to be assigned to a pinning node
	#[pallet::storage]
	#[pallet::getter(fn waiting_ipfs_replicas)]
	#[pallet::unbounded]
	pub type WaitingValidatorIpfsKeys<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ValidatorId, IpfsKeys<T>, ValueQuery>;

	/// Ipfs keys assigned to a pinning node
	#[pallet::storage]
	#[pallet::getter(fn ipfs_replicas)]
	#[pallet::unbounded]
	pub type PinningNodeIpfsKeys<T: Config> =
		StorageMap<_, Blake2_128Concat, PinningNodeIdOf<T>, IpfsKeys<T>, ValueQuery>;

	/// Validator's pinning nodes
	#[pallet::storage]
	#[pallet::getter(fn validators_pinning_nodes)]
	#[pallet::unbounded]
	pub type ValidatorPinningNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ValidatorId, PinningNodes<T>, ValueQuery>;

	/// The ring of the pinning nodes, by means of a circular vector of identifiers (hashes)
	#[pallet::storage]
	#[pallet::getter(fn pinning_nodes_ring)]
	pub type PinningNodesRing<T: Config> = StorageValue<_, PinningRing<T>, ValueQuery>;

	/// Events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]

	pub enum Event<T: Config> {
		/// The replication factor has been set
		ContentReplicationFactorSet {
			factor: ReplicationFactor,
		},
		/// The number of ipfs replicas for each pinning node has changed
		ChangedIpfsReplicasNum {
			ipfs_replicas: u32,
		},
		/// The expected number of pinning nodes per validator has been set
		PinningNodesPerValidatorSet {
			pinning_nodes: u32,
		},
		// A new ipfs node has been registered
		IpfsNodeRegistration {
			validator: T::ValidatorId,
			ipfs_node: T::IPFSNodeId,
		},
		// Pinning node registration
		PinningNodeRegistration {
			validator: T::ValidatorId,
			pinning_node: PinningNodeIdOf<T>,
		},
		/// A pinning node has been removed
		PinningNodeRemoval {
			validator: T::ValidatorId,
			pinning_node: PinningNodeIdOf<T>,
			key_table: KeyTableAt<BlockNumberFor<T>>,
		},
		/// An unassigned ipfs node has been removed
		WaitingIpfsNodeRemoval {
			validator: T::ValidatorId,
			ipfs_node: T::IPFSNodeId,
		},
		/// Unassigned ipfs nodes have been cleared
		WaitingIpfsNodesCleared {
			validator: T::ValidatorId,
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
		/// Ipfs nodes overflow
		IpfsNodesOverflow,
		/// Invalid pinning node
		InvalidPinningNode,
		/// Pinning node already in the ring
		PinningNodeAlreadyInRing,
		/// Too many Pinning nodes in the ring
		RingOutOfBounds,
		// Invalid signature
		InvalidRegistrationSignature,
		/// Ipfs key already in queue
		IpfsKeyAlreadyWaiting,
		/// Ipfs key not found
		IpfsKeyNotFound,
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

		/// Set the number of ipfs replicas per pinning node
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn set_ipfs_replicas(origin: OriginFor<T>, ipfs_replicas: u32) -> DispatchResult {
			// Check that the extrinsic was signed by sudo.
			ensure_root(origin)?;

			ensure!(
				ipfs_replicas.checked_mul(NumOfPinningNodes::<T>::get()).is_some(),
				Error::<T>::IpfsNodesOverflow
			);

			NumOfIpfsReplicas::<T>::put(ipfs_replicas);
			Self::deposit_event(Event::<T>::ChangedIpfsReplicasNum { ipfs_replicas });

			Ok(())
		}

		/// Set the number of pinning nodes associated to each validator
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn set_pinning_nodes_per_validator(
			origin: OriginFor<T>,
			pinning_nodes: u32,
		) -> DispatchResult {
			// Check that the extrinsic was signed by sudo.
			ensure_root(origin)?;

			ensure!(
				pinning_nodes.checked_mul(NumOfIpfsReplicas::<T>::get()).is_some(),
				Error::<T>::IpfsNodesOverflow
			);

			NumOfPinningNodes::<T>::put(pinning_nodes);
			Self::deposit_event(Event::<T>::PinningNodesPerValidatorSet { pinning_nodes });

			Ok(())
		}

		/// A validator registers a new ipfs node
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn register_ipfs_node(
			origin: OriginFor<T>,
			// The registration message signed with the IPFS node key to ensure its owned by the validator
			registration: RegistrationMessageOf<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed by a validator.
			let validator = Self::enure_validator(origin)?;

			let validator_pinning_nodes = ValidatorPinningNodes::<T>::get(&validator).len();
			let ipfs_replicas = NumOfIpfsReplicas::<T>::get() as usize;

			let mut waiting_ipfs_keys = WaitingValidatorIpfsKeys::<T>::get(&validator);

			let total_ipfs_slots =
				validator_pinning_nodes * ipfs_replicas + waiting_ipfs_keys.len();
			ensure!(
				total_ipfs_slots < NumOfPinningNodes::<T>::get() as usize * ipfs_replicas,
				Error::<T>::TooManyIPFSKeys
			);
			ensure!(
				!waiting_ipfs_keys.contains(&registration.key),
				Error::<T>::IpfsKeyAlreadyWaiting
			);

			// We can now add the ipfs key to the unassigned keys of the validator
			waiting_ipfs_keys.push(registration.key.clone());

			// Verify if there are enough ipfs keys to assing to a new pinning node
			let pinning_node = if waiting_ipfs_keys.len() == ipfs_replicas {
				let pinning_id = Self::compute_pinning_node_id(&waiting_ipfs_keys);
				// Insert the new pinning node in the ring.
				// The position is based on the computed id: hash(ipfs_key1||ipfs_key2||...||ipfs_keyN)
				let mut ring = PinningNodesRing::<T>::get();
				let idx = ring
					.binary_search(&pinning_id)
					.err()
					.ok_or(Error::<T>::PinningNodeAlreadyInRing)?;
				ring.try_insert(idx, pinning_id).map_err(|_| Error::<T>::RingOutOfBounds)?;
				// Update the ring and the validator pinning nodes
				PinningNodesRing::<T>::put(ring);
				ValidatorPinningNodes::<T>::mutate(&validator, |nodes| nodes.push(pinning_id));
				// Assign the ipfs keys to the new pinning node (associated to the validator)
				PinningNodeIpfsKeys::<T>::insert(pinning_id, waiting_ipfs_keys);
				// Clear the unassigned ipfs keys
				WaitingValidatorIpfsKeys::<T>::remove(&validator);
				Some(pinning_id)
			} else {
				WaitingValidatorIpfsKeys::<T>::insert(&validator, waiting_ipfs_keys);
				None
			};

			Self::deposit_event(Event::<T>::IpfsNodeRegistration {
				validator: validator.clone(),
				ipfs_node: registration.key.clone(),
			});

			if let Some(pinning_node) = pinning_node {
				Self::deposit_event(Event::<T>::PinningNodeRegistration {
					validator,
					pinning_node,
				});
			}

			Ok(())
		}

		/// Removes a pinning node associated to a validator and all its ipfs replicas
		/// The pinning node sends all its keys (into an encoded version) because they must be managed by a new pinning node
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn rm_pinning_node(
			origin: OriginFor<T>,
			// The position in the ring of the pinning node key to which the ipfs node is assigned
			pinning_node: PinningNodeIdOf<T>,
			// The ipfs cid pointing to the key table of the leaving pinning node for key transfer. The keytable is updated at the block number
			key_table: KeyTableAt<BlockNumberFor<T>>,
		) -> DispatchResult {
			// Check that the extrinsic was signed by a validator.
			let validator = Self::enure_validator(origin)?;

			// Remove the pinning node from the validator's pinning nodes
			ValidatorPinningNodes::<T>::try_mutate(&validator, |nodes| {
				nodes
					.iter()
					.position(|node| node == &pinning_node)
					.map(|pos| nodes.remove(pos))
					.ok_or(Error::<T>::InvalidPinningNode)
			})?;

			// Remove the pinning node from the ring
			let mut ring = PinningNodesRing::<T>::get();
			let idx = ring.binary_search(&pinning_node).expect(
				"Pinning node must be in the ring, because it was found in the validator's pinning nodes",
			);

			ring.remove(idx);
			PinningNodesRing::<T>::put(ring);

			// Remove the ipfs keys associated to the pinning node
			PinningNodeIpfsKeys::<T>::remove(&pinning_node);

			Self::deposit_event(Event::<T>::PinningNodeRemoval {
				validator,
				pinning_node,
				key_table,
			});

			Ok(())
		}

		/// Removes an unassigned ipfs node of a validator
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn rm_waiting_ipfs_node(
			origin: OriginFor<T>,
			ipfs_node: T::IPFSNodeId,
		) -> DispatchResult {
			// Check that the extrinsic was signed by a validator.
			let validator = Self::enure_validator(origin)?;

			let mut waiting_ipfs_keys = WaitingValidatorIpfsKeys::<T>::get(&validator);

			waiting_ipfs_keys
				.iter()
				.position(|key| key == &ipfs_node)
				.map(|pos| {
					waiting_ipfs_keys.remove(pos);
				})
				.ok_or(Error::<T>::IpfsKeyNotFound)?;

			if waiting_ipfs_keys.is_empty() {
				WaitingValidatorIpfsKeys::<T>::remove(&validator);
			} else {
				WaitingValidatorIpfsKeys::<T>::insert(&validator, waiting_ipfs_keys);
			}

			Self::deposit_event(Event::<T>::WaitingIpfsNodeRemoval { validator, ipfs_node });

			Ok(())
		}

		/// Removes all unassigned ipfs nodes of a validator
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(100_000, 0))]
		pub fn clear_waiting_ipfs_nodes(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed by a validator.
			let validator = Self::enure_validator(origin)?;

			WaitingValidatorIpfsKeys::<T>::remove(&validator);

			Self::deposit_event(Event::<T>::WaitingIpfsNodesCleared { validator });

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn compute_pinning_node_id(ipfs_keys: &IpfsKeys<T>) -> PinningNodeIdOf<T> {
		let mut ids = Vec::new();
		ipfs_keys.iter().for_each(|key| ids.extend_from_slice(&key.encode()));

		T::Hashing::hash(&ids[..])
	}

	fn enure_validator(origin: OriginFor<T>) -> Result<T::ValidatorId, DispatchError> {
		let who = ensure_signed(origin)?;
		let validator =
			T::ValidatorIdOf::convert(who).ok_or(Error::<T>::NoAssociatedValidatorId)?;
		ensure!(
			T::ValidatorRegistrar::is_registered(&validator),
			Error::<T>::ValidatorNotRegistered
		);

		Ok(validator)
	}
}
