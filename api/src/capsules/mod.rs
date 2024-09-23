use crate::{
	titanh::{
		self, capsules::calls::types::upload_capsule::App,
		runtime_types::pallet_capsules::capsule::types::CapsuleUploadData,
	},
	TitanhApi,
};
use anyhow::{Ok, Result};
use codec::{Decode, Encode};
use ipfs_api_backend_hyper::{request::Add, IpfsApi, IpfsClient, TryFromUri};
use sp_core::H256;
use std::io::Cursor;
use types::PutCapsuleOpts;

pub struct CapsulesConfig {
	ipfs: IpfsClient,
	app: App,
}

pub struct CapsulesApi<'a> {
	titanh: &'a TitanhApi,
	config: Option<CapsulesConfig>,
}

impl<'a> From<&'a TitanhApi> for CapsulesApi<'a> {
	fn from(titanh: &'a TitanhApi) -> Self {
		CapsulesApi { titanh, config: None }
	}
}

impl<'a> CapsulesApi<'a> {
	/// Provides the IPFS RPC URL and the app id as configuration
	pub fn config(self, ipfs_rpc_url: &str, app: App) -> Result<Self> {
		let ipfs = IpfsClient::from_str(ipfs_rpc_url)?;
		Ok(Self { config: Some(CapsulesConfig { ipfs, app }), ..self })
	}

	/// Put a new object identified by `id` to IPFS and add the metadata to the chain with default options. The transaction is not waited for finalization
	pub async fn put<Id, Value>(&self, id: Id, data: Value) -> Result<H256>
	where
		Id: Encode,
		Value: Encode + Decode,
	{
		let tx_hash = self.put_with_options(id, data, PutCapsuleOpts::default()).await?;
		Ok(tx_hash)
	}

	/// Put a new object identified by `id` to IPFS and add the metadata to the chain, waiting for the transaction to be finalized
	pub async fn put_wait_finalized<Id, Value>(&self, id: Id, data: Value) -> Result<H256>
	where
		Id: Encode,
		Value: Encode + Decode,
	{
		let mut opts = PutCapsuleOpts::default();
		opts.wait_finalization = true;
		let tx_hash = self.put_with_options(id, data, opts).await?;

		Ok(tx_hash)
	}

	/// Put a new object identified by `id` to IPFS and add the metadata to the chain, given the options
	pub async fn put_with_options<Id, Data>(
		&self,
		id: Id,
		data: Data,
		options: PutCapsuleOpts,
	) -> Result<H256>
	where
		Id: Encode,
		Data: Encode,
	{
		// Ensure the configuration is set
		let config = self.ensure_config()?;

		// Encode the data
		let data = Cursor::new(data.encode());
		// Do not pin the data
		let mut add_opts = Add::default();
		add_opts.pin = Some(false);
		// Add the data to IPFS
		let ipfs_res = config.ipfs.add(data).await?;

		let cid = ipfs_res.hash.as_bytes().to_vec();
		let size: u128 =
			ipfs_res.size.parse().expect("Content size is expected to be a valid number");
		let (retention_blocks, followers_status, wait_finalized) =
			options.unwrap_fields_or_default();
		let ending_retention_block = self.titanh.current_block().await? + retention_blocks;

		// Build the capsule
		let capsule = CapsuleUploadData {
			cid,
			size,
			ending_retention_block,
			followers_status,
			encoded_metadata: id.encode(),
		};

		let upload_tx = titanh::tx().capsules().upload_capsule(config.app, None, capsule);

		let tx_hash = if wait_finalized {
			let events = self.titanh.sign_and_submit_wait_finalized(&upload_tx).await?;
			events.extrinsic_hash()
		} else {
			self.titanh.sign_and_submit(&upload_tx).await?
		};

		Ok(tx_hash)
	}
}

pub mod container;
pub mod types;
pub mod utils;
