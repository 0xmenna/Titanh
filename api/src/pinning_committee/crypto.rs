use sp_application_crypto::{Pair, RuntimeAppPublic};

pub mod ed25519 {

	mod app_ed25519 {
		use sp_application_crypto::{app_crypto, ed25519, KeyTypeId};

		const PINNING: KeyTypeId = KeyTypeId(*b"pinn");
		app_crypto!(ed25519, PINNING);
	}

	sp_application_crypto::with_pair! {
		/// An IPFS keypair using ed25519 as its crypto.
		pub type Pair = app_ed25519::Pair;
	}
}

pub enum KeyError {
	InvalidSeedLength,
}

pub struct IpfsPair(ed25519::Pair);

impl IpfsPair {
	pub fn from_seed(seed: &[u8]) -> Result<Self, KeyError> {
		let pair = ed25519::Pair::from_seed_slice(seed).map_err(|_| KeyError::InvalidSeedLength)?;
		Ok(Self(pair))
	}

	pub fn public(&self) -> Vec<u8> {
		self.0.public().to_raw_vec()
	}

	pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
		self.0.sign(msg).to_vec()
	}
}
