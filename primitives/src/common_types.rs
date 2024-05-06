use codec::{Codec, Decode, Encode, MaxEncodedLen};
pub use frame_support::traits::Time;
use frame_support::Parameter;
pub use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member},
	BoundedVec, FixedPointOperand,
};
use sp_std::fmt::Debug;
use sp_std::vec::Vec;
pub enum Error {
	ScaleCodecDecodeError,
	BadEncodedData,
	BadString,
}

// Trait in which we record balances
pub trait Balance:
	Parameter
	+ Member
	+ AtLeast32BitUnsigned
	+ Codec
	+ Default
	+ Copy
	+ MaybeSerializeDeserialize
	+ Debug
	+ MaxEncodedLen
	+ TypeInfo
	+ FixedPointOperand
{
}

/// A type for representing an IPFS CID
pub type CidFor<T> = HashOf<T>;

/// The type in which the chain records hashes
pub type HashOf<T> = <T as frame_system::Config>::Hash;

/// A bounded vector of te chain's accounts (aka public keys)
pub type Accounts<AccountId, S> = BoundedVec<AccountId, S>;

pub type ContentSize = u128;

#[derive(Encode, Decode, MaxEncodedLen, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(S))]
pub struct EncodedData<S: Get<u32>>(BoundedVec<u8, S>);

impl<S: Get<u32>> EncodedData<S> {
	pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
		let res = data.to_vec().try_into().map_err(|_| Error::BadEncodedData)?;

		Ok(Self(res))
	}

	pub fn from_plain_data<E: Encode>(data: E) -> Result<Self, Error> {
		let data = data.encode();
		Self::from_slice(&data)
	}

	pub fn decode<D: Decode>(&self) -> Result<D, Error> {
		Decode::decode(&mut &self.0[..]).map_err(|_| Error::ScaleCodecDecodeError)
	}

	pub fn as_slice(&self) -> &[u8] {
		self.0.as_slice()
	}

	pub fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}
}

/// A bounded string
#[derive(Encode, Decode, MaxEncodedLen, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
#[scale_info(skip_type_params(S))]
pub struct BoundedString<S: Get<u32>>(BoundedVec<u8, S>);

impl<S: Get<u32>> BoundedString<S> {
	pub fn from_vec(str_vec: Vec<u8>) -> Result<Self, Error> {
		let str = str_vec.try_into().map_err(|_| Error::BadString)?;

		Ok(Self(str))
	}

	pub fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}
}
