use crate::types::{
	chain::{titanh, BlockHash, NodeId, Rpc, Signer, SubstrateApi},
	pinning::{PinningCapsuleEvent, PinningRing},
};
use anyhow::Result;
use primitives::BlockNumber;
use std::rc::Rc;
use subxt::{storage::Address, utils::Yes};

/// Substrate client with a default configuration
/// It handles chain state requests and transactions
#[derive(Clone)]
pub struct SubstrateClient {
	/// The Substrate api to query the chain storage
	api: SubstrateApi,
	/// The chain rpc methods
	rpc: Rpc,
	/// The singer of transactions
	signer: Signer,
	/// The node id bounded to the client
	node_id: NodeId,
	/// A reference to the pinning ring
	pinning_ring: Rc<PinningRing>,
}

impl SubstrateClient {
	pub fn new(
		api: SubstrateApi,
		rpc: Rpc,
		signer: Signer,
		node_id: NodeId,
		pinning_ring: Rc<PinningRing>,
	) -> Self {
		SubstrateClient { api, rpc, signer, node_id, pinning_ring }
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

	/// Given a block hash, it returns the list of pinning capsule events that are relevant to the pinning node, based on the pinning ring.
	async fn pinning_events_at(&self, block_hash: BlockHash) -> Result<Vec<PinningCapsuleEvent>> {
		let events_query = titanh::storage().system().events();
		// Events at block identified by `block_hash`
		let events = self.query(&events_query, Some(block_hash)).await?;

		let mut pinning_events = Vec::new();
		for event_record in events.into_iter() {
			let event = PinningCapsuleEvent::try_from_runtime_event(event_record.event);

			if let Some(event) = event {
				let is_node_replica =
					self.pinning_ring.is_key_owned_by_node(event.key, self.node_id)?;

				if is_node_replica {
					pinning_events.push(event)
				}
			}
		}

		Ok(pinning_events)
	}

	/// Returns the list of pinning capsule events occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
	pub async fn pinning_events_in_range(
		&self,
		start: BlockNumber,
		end: BlockNumber,
		skip_num_events: usize,
	) -> Result<Vec<PinningCapsuleEvent>> {
		let mut capsule_events = Vec::new();
		for block_number in start..=end {
			let block_hash = self.get_block_hash(block_number).await?;

			let mut events = self.pinning_events_at(block_hash).await?;

			if block_number == start {
				// remove first `skip_num_events`
				events.drain(0..skip_num_events);
			}

			capsule_events.extend(events);
		}

		Ok(capsule_events)
	}

	/// Returns the block hash of a n associated block number
	async fn get_block_hash(&self, block_number: BlockNumber) -> Result<BlockHash> {
		let block_hash_query = titanh::storage().system().block_hash(&block_number);
		let block_hash = self.query(&block_hash_query, None).await?;

		Ok(block_hash.into())
	}
}
