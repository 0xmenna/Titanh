use super::{batch::Batch, KeyedPinningEvent, NodeEvent};
use crate::{
	db::checkpointing::{CheckpointEvent, DbCheckpoint as DbClient},
	ipfs::{self, client::IpfsClient},
	substrate::client2::SubstratePinningClient,
	types::keytable::{FaultTolerantKeyTable, KeyTable, OrderedRows, Row},
	utils::{
		ref_builder::{MutableRef, Ref},
		traits::{Dispatcher, MutableDispatcher},
	},
};
use anyhow::Result;
use api::{
	common_types::BlockNumber,
	pinning_committee_types::{NodeId, PinningRing},
};
use async_trait::async_trait;
use codec::Decode;
use futures::stream::ForEach;

/// Event dispatcher
pub struct EventDispatcher {
	/// Dispatcher for the database (checkpointing)
	db: Ref<DbClient>,
	/// Dispatcher for the IPFS operations
	ipfs: MutableRef<IpfsClient>,
	/// Dispatcher for the the chain operations
	substrate: MutableRef<SubstratePinningClient>,
	/// The block number until which the node has checkpointed the processed events.
	block_num: BlockNumber,
}

#[async_trait(?Send)]
impl MutableDispatcher<Batch<NodeEvent>, ()> for EventDispatcher {
	async fn dispatch(&mut self, batch: Batch<NodeEvent>) -> Result<()> {
		// These replay blocks are the ones that may be replayed because of a node leave event. The node that leaves the ring transfers its keys up to date only to a certain block number, so the pinning node must replay events from the block number after to the current block number.
		for (idx, event) in batch.into_iter().enumerate() {
			// Handle event
			match event {
				// Pinning event
				NodeEvent::Pinning(event) => {
					let maybe_pin = self.substrate.borrow_mut().dispatch(event).await?;

					if let Some(pin_event) = maybe_pin {
						// Pinning dispatch
						self.ipfs.borrow_mut().dispatch(pin_event).await.unwrap();
					}
				},
				// Node registration event
				NodeEvent::NodeRegistration(node_id) => {
					self.substrate.borrow_mut().dispatch(node_id).await?;
				},
				// Node removal event
				NodeEvent::NodeRemoval(leave_event) => {
					// Dispatch the leave event and get the CID that locates the row to be transferred
					let res = self.substrate.borrow_mut().dispatch(leave_event).await?;
					if let Some((dist_from_node, cid)) = res {
						let transferred_row = self.ipfs.borrow_mut().get(cid).await?;
						let mut row = Row::decode(&mut &transferred_row[..])?;
						self.substrate.borrow_mut().mutable_keytable().extend_last_row(&mut row);

						// recover keys not transferred from leaving node. These range from the block after the one that indentifies the keytable version and the current block number being processed (excluding events that still need to be processed, since these will be captured).
						debug_assert!(leave_event.key_table_at() <= self.block_num);

						let raplay = ReplayBlocks::new(
							ReplayCause::new(leave_event.node(), dist_from_node),
							leave_event.key_table_at() + 1,
							(self.block_num + 1, idx),
						);
						let replay_batch = self.substrate.borrow().replay_blocks(raplay).await?;

						for pin in replay_batch {
							self.ipfs.borrow_mut().dispatch(pin).await.unwrap();
						}
					}
				},
				// Block barrier event for checkpointing
				NodeEvent::BlockBarrier(block_num) => {
					// get the rows of the keytable to be flushed
					let flushing_rows = self.substrate.borrow_mut().mutable_keytable().flush();

					// commit the checkpoint
					let checkpoint_event = CheckpointEvent::new(block_num, flushing_rows);
					self.db.dispatch(checkpoint_event).await?;

					// update the block number
					self.block_num = block_num;
				},
			};
		}

		Ok(())
	}
}

pub struct ReplayBlocks {
	cause: ReplayCause,
	from: BlockNumber,
	// block number and event index within the block
	to: (BlockNumber, usize),
}

impl ReplayBlocks {
	pub fn new(cause: ReplayCause, from: BlockNumber, to: (BlockNumber, usize)) -> Self {
		ReplayBlocks { cause, from, to }
	}

	pub fn cause(&self) -> &ReplayCause {
		&self.cause
	}

	pub fn from(&self) -> BlockNumber {
		self.from
	}

	pub fn to(&self) -> (BlockNumber, usize) {
		self.to
	}
}

pub struct ReplayCause {
	pub node: NodeId,
	pub dist_from_node: u32,
}

impl ReplayCause {
	pub fn new(node: NodeId, dist_from_node: u32) -> Self {
		ReplayCause { node, dist_from_node }
	}
}
