use crate::types::chain::{BlockHash, CapsuleEvents};
use crate::types::chain::{Rpc, Signer, SubstrateApi};
use anyhow::{Context, Result};
use primitives::BlockNumber;
use std::sync::Arc;
use std::thread;
use subxt::{storage::Address, utils::Yes};
use titanh::capsules::events::CapsuleUploaded;
use tokio::sync::Mutex;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod titanh {}

/// Substrate client with a default configuration
/// It handles chain state requests and transactions
#[derive(Clone)]
pub struct SubstrateClient {
	api: SubstrateApi,
	rpc: Rpc,
	signer: Signer,
}

impl SubstrateClient {
	pub fn new(api: SubstrateApi, rpc: Rpc, signer: Signer) -> Self {
		SubstrateClient { api, rpc, signer }
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
		let storage_client = self.api.storage();

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

	pub async fn handle_capsule_events(&self) -> Result<()> {
		let capsule_events = Arc::new(Mutex::new(Vec::<CapsuleEvents>::new()));

		self.subscribe_to_chain_events().await
	}

	async fn subscribe_to_chain_events(&self) -> Result<()> {
		let mut blocks_sub = self.api.blocks().subscribe_finalized().await?;
		while let Some(block) = blocks_sub.next().await {
			let block = block?;

			let events = block.events().await?;
			for event in events.iter() {
				let event = event?;

				if let Some(upload_event) = event.as_event::<CapsuleUploaded>()? {
					println!("Capsule uploaded: {:?}", upload_event);
				};
			}
		}

		Ok(())
	}

	pub async fn capsule_events_from_block(
		&self,
		block_number: BlockNumber,
	) -> Result<Vec<CapsuleEvents>> {
		let block_query = titanh::storage().system().block_hash(block_number);
		let at = self.query(&block_query, None).await?;

		let events_query = titanh::storage().system().events();

		let result = self.query(&events_query, Some(at.into())).await?;

		let mut events = Vec::new();
		result.into_iter().for_each(|record| {
			let event = record.event;

			// if let CapsuleUploaded {} = event {}
		});

		Ok(events)
	}
}
