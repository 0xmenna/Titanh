use crate::types::{
	chain::{
		titanh::{self},
		BlockHash, NodeId, Rpc, Signer, SubstrateApi,
	},
	events::{self, NodeEvent},
	ring::PinningRing,
};
use anyhow::Result;
use primitives::BlockNumber;
use std::sync::Arc;
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
	pinning_ring: Arc<PinningRing>,
}

impl SubstrateClient {
	pub fn new(
		api: SubstrateApi,
		rpc: Rpc,
		signer: Signer,
		node_id: NodeId,
		pinning_ring: Arc<PinningRing>,
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

	/// Given a block hash, it returns the list of events that are relevant to the pinning node, based on the pinning ring.
	pub async fn events_at(&self, block_hash: BlockHash) -> Result<Vec<NodeEvent>> {
		let events_query = titanh::storage().system().events();
		// Events at block identified by `block_hash`
		let events = self.query(&events_query, Some(block_hash)).await?;

		let mut pinning_events = Vec::new();
		for event_record in events.into_iter() {
			let event = events::try_pinning_event_from_runtime(event_record.event);

			if let Some(event) = event {
				let is_node_replica =
					self.pinning_ring.is_key_owned_by_node(event.key, self.node_id)?;

				if is_node_replica {
					pinning_events.push(event.into())
				}
			}
		}

		Ok(pinning_events)
	}

	/// Returns the list of pinning capsule events occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
	pub async fn events_in_range(
		&self,
		start: BlockNumber,
		end: BlockNumber,
	) -> Result<Vec<NodeEvent>> {
		let mut capsule_events = Vec::new();
		for block_number in start..=end {
			let block_hash = self.get_block_hash(block_number).await?;

			let events = self.events_at(block_hash).await?;
			capsule_events.extend(events);
			// Add barrier event for later checkpointing
			capsule_events.push(NodeEvent::BlockCheckpoint(block_number));
		}

		Ok(capsule_events)
	}

	/// Returns the block hash of a n associated block number
	async fn get_block_hash(&self, block_number: BlockNumber) -> Result<BlockHash> {
		let block_hash_query = titanh::storage().system().block_hash(&block_number);
		let block_hash = self.query(&block_hash_query, None).await?;

		Ok(block_hash.into())
	}

	// Returns the state of the ring
	pub async fn get_ring_state(&self) -> Result<PinningRing> {
		let ring_state_query = titanh::storage().pinning_committee().pinning_nodes_ring();
		let hash_nodes_bounded = self.query(&ring_state_query, None).await?;
		let hash_nodes = hash_nodes_bounded.0.to_vec();
		let replication_factor_query =
			titanh::storage().pinning_committee().content_replication_factor();
		let replication_factor = self.query(&replication_factor_query, None).await?;
		let nodes_in_ring: PinningRing = PinningRing::new(hash_nodes, replication_factor);
		Ok(nodes_in_ring)
	}

	pub fn get_api(&self) -> &SubstrateApi {
		&self.api
	}
}
