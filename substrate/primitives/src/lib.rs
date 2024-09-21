//! Low-level types used throughout the Substrate stencil code.

#![cfg_attr(not(feature = "std"), no_std)]

mod common_types;

pub use common_types::*;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	KeyTypeId, MultiSignature, OpaqueExtrinsic as UncheckedExtrinsic,
};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Block header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Block identifier type.
pub type BlockId = generic::BlockId<Block>;
/// Session index
pub type SessionIndex = u32;

pub const PINNING: KeyTypeId = KeyTypeId(*b"pinn");

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 3000;

// We agreed to 5MB as the block size limit.
pub const MAX_BLOCK_SIZE: u32 = 5 * 1024 * 1024;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Session duration (approximately 6 hours)
pub const DEFAULT_SESSION_PERIOD: u32 = 7200;

pub const TOKEN_DECIMALS: u32 = 12;
pub const TOKEN: u128 = 10u128.pow(TOKEN_DECIMALS);

pub const ADDRESSES_ENCODING: u8 = 42;
