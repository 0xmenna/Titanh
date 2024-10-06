use crate::{
    db::checkpointing::DbCheckpoint as DbDispatcher,
    ipfs::client::IpfsClient as PinDispatcher,
    substrate::client::SubstrateClient,
    types::{
        batch::Batch,
        events::{CheckpointEvent, NodeEvent},
        keytable::FaultTolerantKeyTable,
    },
    utils::ref_builder::AtomicRef,
};
use anyhow::Result;
use api::{common_types::BlockNumber, pinning_committee_types::PinningRing};
use async_trait::async_trait;
use keys_dispatcher::KeysDispatcher;
use traits::{AsyncMutableDispatcher, Dispatcher, MutableDispatcher};

/// Event dispatcher
pub struct NodeEventDispatcher {
    /// Dispatcher for the database (checkpointing)
    db: DbDispatcher,
    /// Dispatcher for pinning operations on IPFS
    pinning: PinDispatcher,
    /// Dispatcher for capsule keys operations
    keys: KeysDispatcher,
    /// The block number until which the node has checkpointed the processed events.
    block_num: BlockNumber,
}

impl NodeEventDispatcher {
    pub fn from_config(
        db: DbDispatcher,
        pin: PinDispatcher,
        sub_client: AtomicRef<SubstrateClient>,
        ring: PinningRing,
        keytable: FaultTolerantKeyTable,
        block_num: BlockNumber,
    ) -> Self {
        let keys: KeysDispatcher = KeysDispatcher::new(sub_client, ring, keytable);

        Self {
            db,
            pinning: pin,
            keys,
            block_num,
        }
    }
}

#[async_trait(?Send)]
impl AsyncMutableDispatcher<Batch<NodeEvent>, ()> for NodeEventDispatcher {
    async fn async_dispatch(&mut self, batch: Batch<NodeEvent>) -> Result<()> {
        for (idx, event) in batch.into_iter().enumerate() {
            // Handle event
            match event {
                // Pinning event
                NodeEvent::Pinning(event) => {
                    log::info!("Dispatching pinning event {:?}", event.clone());
                    let maybe_pin = self.keys.dispatch(event)?;

                    if let Some(pin_event) = maybe_pin {
                        // Pinning dispatch
                        self.pinning
                            .async_dispatch(pin_event.clone())
                            .await
                            .unwrap();
                        log::info!("Pinning event dispatched successfully");
                    }
                }
                // Node registration event
                NodeEvent::NodeRegistration(node_id) => {
                    log::info!(
                        "Dispatching node registration event. New node ID{:?}",
                        node_id
                    );
                    // Removes the keys that will be handled by the new node (if any)
                    let rm_row = self.keys.dispatch(node_id)?;
                    if let Some(unpinning_event) = rm_row {
                        // unpin the cids in the removed row
                        self.pinning.async_dispatch(unpinning_event).await.unwrap()
                    }
                    log::info!("Node registration event dispatched successfully");
                }
                // Node removal event
                NodeEvent::NodeRemoval(leave_event) => {
                    log::info!("Dispatching node removal event {:?}", leave_event);
                    // (event, event_block_num, event_idx)
                    let leaved_event_at = (leave_event, self.block_num + 1, idx);
                    // Dispatch the leave event and get the CID that locates the row to be transferred
                    let res = self.keys.async_dispatch(leaved_event_at).await?;
                    if let Some((cid, batch)) = res {
                        println!("CID of row to recover: {:?}", cid);
                        let mut transferred_row = self.pinning.async_dispatch((cid, batch)).await?;
                        // Update the table with the row fetched from IPFS
                        self.keys
                            .mutable_keytable()
                            .extend_last_row(&mut transferred_row)?;
                    }
                    log::info!("Node removal event dispatched successfully");
                }
                // Block barrier event for checkpointing
                NodeEvent::BlockBarrier(block_num) => {
                    log::info!("Checkpointing at block_num {:?}", block_num);
                    // get the rows of the keytable to be flushed
                    let flushing_rows = self.keys.mutable_keytable().flush();

                    // commit the checkpoint
                    let checkpoint_event = CheckpointEvent::new(block_num, flushing_rows);
                    self.db.dispatch(checkpoint_event)?;

                    // update the block number
                    self.block_num = block_num;

                    // Log the keytable if needed
                    self.keys.keytable().log(block_num)?;
                }
            };
        }

        Ok(())
    }
}

pub mod db_dispatcher;
pub mod keys_dispatcher;
pub mod pin_dispatcher;
pub mod traits;
