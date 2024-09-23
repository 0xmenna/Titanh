use super::{types::PutCapsuleOpts, CapsulesApi, CapsulesConfig};
use crate::titanh::{
	self, capsules::calls::types::UploadCapsule,
	runtime_types::pallet_capsules::capsule::types::CapsuleUploadData,
};
use anyhow::Result;
use codec::Encode;
use ipfs_api_backend_hyper::{request::Add, IpfsApi};
use std::io::Cursor;
use subxt::tx::DefaultPayload;

impl CapsulesApi<'_> {
	// ensures the api configuration is set
	pub fn ensure_config(&self) -> Result<&CapsulesConfig> {
		self.config
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Capsules API configuration is not provided"))
	}
}
