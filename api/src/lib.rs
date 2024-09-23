use anyhow::Result;
use app_registrar::AppRegistrarApi;
use capsules::CapsulesApi;
use common::{
	titanh::Call,
	types::{BlockHash, BlockNumber, Rpc, Signer, SubstrateApi},
};
use pinning_committee::PinningCommitteeApi;
use sp_core::H256;
use subxt::{blocks::ExtrinsicEvents, storage::Address, tx::Payload, utils::Yes, SubstrateConfig};

mod app_registrar;
mod builder;
mod capsules;
mod common;
mod pinning_committee;

pub use builder::TitanhApiBuilder;
pub use common::{titanh, types};

/// Titanh api
#[derive(Clone)]
pub struct TitanhApi {
	/// The Substrate api to query the chain storage
	pub substrate_api: SubstrateApi,
	/// The chain rpc methods
	pub rpc: Rpc,
	/// The singer of transactions
	pub signer: Option<Signer>,
}

impl TitanhApi {
	pub fn new(substrate_api: SubstrateApi, rpc: Rpc, signer: Option<Signer>) -> Self {
		TitanhApi { substrate_api, rpc, signer }
	}

	/// Queries the chain's storage
	pub async fn query<'address, Addr>(
		&self,
		address: &'address Addr,
		at: Option<BlockHash>,
	) -> Result<<Addr as Address>::Target>
	where
		Addr: Address<IsFetchable = Yes> + 'address,
	{
		let storage_client = self.substrate_api.storage();

		let storage = if let Some(block_hash) = at {
			storage_client.at(block_hash)
		} else {
			storage_client.at_latest().await?
		};

		// This returns an `Option<_>`, which will be
		// `None` if no value exists at the given address.
		let result = storage
			.fetch(address)
			.await?
			.ok_or_else(|| anyhow::anyhow!("Vale is not defined in storage"))?;
		Ok(result)
	}

	// Returns the state of the ring
	// pub async fn ring_state(&self) -> Result<PinningRing> {
	// 	let ring_state_query = titanh::storage().pinning_committee().pinning_nodes_ring();
	// 	let hash_nodes_bounded = self.query(&ring_state_query, None).await?;
	// 	let hash_nodes = hash_nodes_bounded.0.to_vec();
	// 	let replication_factor_query =
	// 		titanh::storage().pinning_committee().content_replication_factor();
	// 	let replication_factor = self.query(&replication_factor_query, None).await?;
	// 	let nodes_in_ring: PinningRing = PinningRing::new(hash_nodes, replication_factor);
	// 	Ok(nodes_in_ring)
	// }

	/// Returns the block hash of a n associated block number
	async fn block_hash(&self, block_number: BlockNumber) -> Result<BlockHash> {
		let block_hash_query = titanh::storage().system().block_hash(&block_number);
		let block_hash = self.query(&block_hash_query, None).await?;

		Ok(block_hash.into())
	}

	pub async fn current_block(&self) -> Result<BlockNumber> {
		let block = self.rpc.chain_get_block(None).await?.unwrap();
		Ok(block.block.header.number)
	}

	fn ensure_signer(&self) -> Result<&Signer> {
		self.signer.as_ref().ok_or_else(|| anyhow::anyhow!("Signer is not set"))
	}

	/// Signs and submits a transaction. If it succeeds, it means the transaction is included in the transaction pool, not in a block.
	pub async fn sign_and_submit<Call: Payload>(&self, tx: &Call) -> Result<H256> {
		let signer = self.ensure_signer()?;
		let tx_hash = self.substrate_api.tx().sign_and_submit_default(tx, signer).await?;

		Ok(tx_hash)
	}

	/// Signs and submits a transaction. It waits until the transaction is finalized.
	pub async fn sign_and_submit_wait_finalized<Call: Payload>(
		&self,
		tx: &Call,
	) -> Result<ExtrinsicEvents<SubstrateConfig>> {
		let signer = self.ensure_signer()?;

		// Submit the extrinisc, and wait for it to be successful and in a finalized block.
		// We get back the extrinsic events if all is well.
		let events = self
			.substrate_api
			.tx()
			.sign_and_submit_then_watch_default(tx, signer)
			.await?
			.wait_for_finalized_success()
			.await?;

		Ok(events)
	}

	/// Returns the app registrar api
	pub fn app_registrar(&self) -> AppRegistrarApi<'_> {
		AppRegistrarApi::from(self)
	}

	/// Returns the capsules api
	pub fn capsules(&self) -> CapsulesApi<'_> {
		CapsulesApi::from(self)
	}

	/// Returns the pinning committee api
	pub fn pinning_committee(&self) -> PinningCommitteeApi<'_> {
		PinningCommitteeApi::from(self)
	}
}
