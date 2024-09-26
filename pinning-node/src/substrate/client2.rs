use std::borrow::BorrowMut;

use crate::{
	types::{
		cid::Cid,
		events::{
			self, batch::Batch, dispatcher::ReplayBlocks, BlockBarrierEvent, JoinNodeEvent,
			KeyedPinningEvent, LeaveNodeEvent, NodeEvent, PinningEvent,
		},
		keytable::{FaultTolerantKeyTable, Row},
	},
	utils::traits::MutableDispatcher,
};
use anyhow::Result;
use api::{
	capsules_types::CapsuleKey,
	common_types::{BlockHash, BlockInfo, BlockNumber},
	pinning_committee_types::{NodeId, PinningRing},
	TitanhApi,
};
use async_trait::async_trait;

pub struct SubstratePinningClient {
	api: TitanhApi,
	/// The node id bounded to the client
	node_id: NodeId,
	/// The block at which the client has been intialized
	block: BlockInfo,
	/// The pinning node's ring
	ring: PinningRing,
	/// The table of keys that the node is responsible for.
	keytable: FaultTolerantKeyTable,
}

impl SubstratePinningClient {
	pub fn new(
		api: TitanhApi,
		node_id: NodeId,
		block: BlockInfo,
		ring: PinningRing,
		keytable: FaultTolerantKeyTable,
	) -> Self {
		SubstratePinningClient { api, node_id, block, ring, keytable }
	}

	/// Return the events of a given block, and returns the list of events that are relevant to the pinning node, based on the pinning ring.
	pub async fn events_at(&self, block: BlockInfo) -> Result<Batch<NodeEvent>> {
		// Events at block identified by `block_hash`
		let runtime_events = self.api.runtime_events(Some(block.hash)).await?;

		let mut batch = Batch::default();
		for event_record in runtime_events.into_iter() {
			let node_event = events::try_event_from_runtime(event_record.event);
			if let Some(event) = node_event {
				if event.is_committee_event() && self.block.number >= block.number {
					// The client has been initialized at a later block, so we ignore if the event is a committee event (join or leave). This is because the node boudend to the client has a most up to date ring.
					continue;
				}
				batch.insert(event);
			}
		}

		// Add a block barrier event for later checkpointing
		batch.insert(NodeEvent::BlockBarrier(block.number));

		Ok(batch)
	}

	/// Returns the list of events occured between a block range. It can skip a number of events for the `start` block because they may have been already processed.
	pub async fn events_in_range(
		&self,
		start: BlockNumber,
		end: BlockNumber,
	) -> Result<Batch<NodeEvent>> {
		let mut batch = Batch::default();

		for block_number in start..=end {
			let block =
				BlockInfo { number: block_number, hash: self.api.block_hash(block_number).await? };

			let block_batch = self.events_at(block).await?;
			batch.extend(block_batch);
		}

		Ok(batch)
	}

	pub async fn pinning_events_at(
		&self,
		at: BlockHash,
		break_event_idx: usize,
	) -> Result<Batch<KeyedPinningEvent>> {
		let runtime_events = self.api.runtime_events(Some(at)).await?;

		let mut batch = Batch::default();
		for (index, event_record) in runtime_events.into_iter().enumerate() {
			if index == break_event_idx {
				break;
			}
			let node_event = events::try_event_from_runtime(event_record.event);
			if let Some(event) = node_event {
				event.pinning_event().map(|event| batch.insert(event));
			}
		}

		Ok(batch)
	}

	pub fn api(&self) -> &TitanhApi {
		&self.api
	}

	pub fn node_id(&self) -> NodeId {
		self.node_id
	}
}

impl SubstratePinningClient {
	pub fn prepare_dispatcher(&self) {}
}

/// Dispatcher for processing pinning events (events related to capsules)
#[async_trait(?Send)]
impl MutableDispatcher<KeyedPinningEvent, Option<PinningEvent>> for SubstratePinningClient {
	async fn dispatch(&mut self, event: KeyedPinningEvent) -> Result<Option<PinningEvent>> {
		// If the pinning node is responsible for the key, then we get the partition number to which the key belongs. Else, we ignore the event.
		let maybe_partition = self.ring.key_node_partition(event.key, self.node_id)?;

		let pin = if let Some(partition) = maybe_partition {
			// The key belongs to the pinning node

			// Update the keytable
			let row_idx = partition.saturating_sub(1);
			match event.pin {
				PinningEvent::Pin { cid } => {
					self.keytable.insert(row_idx, event.key, cid)?;
				},
				PinningEvent::RemovePin { .. } => {
					self.keytable.remove(row_idx, &event.key)?;
				},
				PinningEvent::UpdatePin { new_cid, .. } => {
					self.keytable.insert(row_idx, event.key, new_cid)?;
				},
			}

			Some(event.pin)
		} else {
			None
		};

		Ok(pin)
	}
}

impl SubstratePinningClient {
	pub fn mutable_keytable(&mut self) -> &mut FaultTolerantKeyTable {
		&mut self.keytable
	}
}

impl SubstratePinningClient {
	pub async fn replay_blocks(&self, replay: ReplayBlocks) -> Result<Batch<PinningEvent>> {
		// Replay the events from the replay blocks
		let cause = replay.cause();
		let from_block = replay.from();
		let (to_block, break_event_idx) = replay.to();

		let mut pinning_batch = Batch::default();
		for block_num in from_block..=to_block {
			let block_hash = self.api.block_hash(block_num).await?;

			let batch = self.events_at(BlockInfo::new(block_num, block_hash)).await?;

			for (idx, event) in batch.into_iter().enumerate() {
				// stop at the break event
				if block_num == to_block && idx == break_event_idx {
					break;
				}

				let maybe_pinning_event = event.pinning_event();
				if let Some(keyed_event) = maybe_pinning_event {
					if self.is_key_owned_at_replay(
						keyed_event.key,
						cause.dist_from_node,
						cause.node,
					)? {
						pinning_batch.insert(keyed_event.pin);
					}
				}
			}
		}

		Ok(pinning_batch)
	}

	fn is_key_owned_at_replay(
		&self,
		key: CapsuleKey,
		dist_from_node: u32,
		node: NodeId,
	) -> Result<bool> {
		let maybe_partition = self.ring.key_node_partition(key, self.node_id)?;

		if let Some(partition) = maybe_partition {
			if partition as u32 == self.ring.replication() {
				if dist_from_node == self.ring.replication() && key >= node {
					return Ok(false);
				}
			}
			return Ok(true);
		}

		return Ok(false);
	}
}

/// Dispatcher for processing a node join event
#[async_trait(?Send)]
impl MutableDispatcher<JoinNodeEvent, ()> for SubstratePinningClient {
	async fn dispatch(&mut self, node: JoinNodeEvent) -> Result<()> {
		let idx = self.ring.insert_node(&node)?;

		let dist = self.ring.distance_from_idx(idx, &self.node_id)?;
		if dist <= self.ring.replication() {
			// The pinning node is impacted by the join, so it should drop some keys.
			// First, the node select which row must me partitioned in two.
			// Then, it puts the new row resulting from the partitioning to the first position, and shifts the other rows, by also deleting the last one.
			let row_idx = dist as usize - 1;
			let partition_barrier = node;
			self.keytable.partition_row(row_idx, &partition_barrier)?;
		}

		Ok(())
	}
}

/// Dispatcher for processing a node leave event. It returns the IPFS cid that the node must fetch in order to get the row to update the keytable (if any).
#[async_trait(?Send)]
impl<'a> MutableDispatcher<LeaveNodeEvent, Option<(u32, Cid)>> for SubstratePinningClient {
	async fn dispatch(&mut self, leave_event: LeaveNodeEvent) -> Result<Option<(u32, Cid)>> {
		let left_node = leave_event.node();

		let dist = self.ring.distance_between(&self.node_id, &left_node)?;
		self.ring.remove_node(&left_node)?;

		if dist <= self.ring.replication() {
			// The row that needs to be merged with the next one
			let row_idx = dist as usize - 1;
			self.keytable.merge_rows_from(row_idx)?;

			// The IPFS cid of interest containing the row to fetch from IPFS from the left node that needs to be added as the last row of the keytable
			let cid_idx = self.ring.replication() - dist;
			let cid = leave_event.row_cid_of(cid_idx as usize);

			return Ok(Some((dist, cid)));
		}

		Ok(None)
	}
}
