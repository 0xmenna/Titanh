use super::traits::{AsyncMutableDispatcher, MutableDispatcher};
use crate::{
    substrate::client::SubstrateClient,
    types::{
        batch::Batch,
        cid::Cid,
        events::{JoinNodeEvent, KeyedPinningEvent, LeaveNodeEventAt, PinningEvent},
        keytable::FaultTolerantKeyTable,
    },
    utils::ref_builder::AtomicRef,
};
use anyhow::Result;
use api::{
    capsules_types::CapsuleKey,
    common_types::{BlockInfo, BlockNumber},
    pinning_committee_types::{NodeId, PinningRing},
};
use async_trait::async_trait;

pub struct KeysDispatcher {
    client: AtomicRef<SubstrateClient>,
    /// The pinning node's ring
    ring: PinningRing,
    /// The table of keys that the node is responsible for.
    keytable: FaultTolerantKeyTable,
}

impl KeysDispatcher {
    pub fn new(
        client: AtomicRef<SubstrateClient>,
        ring: PinningRing,
        keytable: FaultTolerantKeyTable,
    ) -> Self {
        KeysDispatcher {
            client,
            ring,
            keytable,
        }
    }
}

/// Dispatcher for processing pinning events (events related to capsules)
impl MutableDispatcher<KeyedPinningEvent, Option<PinningEvent>> for KeysDispatcher {
    fn dispatch(&mut self, event: KeyedPinningEvent) -> Result<Option<PinningEvent>> {
        // If the pinning node is responsible for the key, then we get the partition number to which the key belongs. Else, we ignore the event.
        let maybe_partition = self
            .ring
            .key_node_partition(event.key, self.client.node_id())?;

        let pin = if let Some(partition_idx) = maybe_partition {
            // The key belongs to the pinning node

            // Update the keytable
            self.update_table_from_event(partition_idx, &event)?;

            Some(event.pin)
        } else {
            None
        };

        Ok(pin)
    }
}

/// Dispatcher for processing a node join event
impl MutableDispatcher<JoinNodeEvent, ()> for KeysDispatcher {
    fn dispatch(&mut self, node: JoinNodeEvent) -> Result<()> {
        // Insert the node and retrieve its position in the ring
        let idx = self.ring.insert_node(&node)?;
        // Get the distance of `self` with respect to the new node
        let dist = self.ring.distance_from_idx(idx, &self.client.node_id())?;
        if dist <= self.ring.replication() {
            // The pinning node is impacted by the join, so it should drop some keys.
            // First, the node selects which row must me partitioned in two.
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
impl<'a> AsyncMutableDispatcher<LeaveNodeEventAt, Option<(Cid, Batch<PinningEvent>)>>
    for KeysDispatcher
{
    async fn async_dispatch(
        &mut self,
        leave_event_at: LeaveNodeEventAt,
    ) -> Result<Option<(Cid, Batch<PinningEvent>)>> {
        let (leave_event, event_block_num, event_idx) = leave_event_at;

        let left_node = leave_event.node();
        let dist = self
            .ring
            .distance_between(&self.client.node_id(), &left_node)?;
        self.ring.remove_node(&left_node)?;

        if dist == 0 {
            panic!("The current node is not expected to read a leave event of itself, remove the node checkpointing db and restart the node");
        }

        if dist <= self.ring.replication() {
            // The row that needs to be merged with the next one
            let row_idx = dist as usize - 1;
            self.keytable.merge_rows_from(row_idx)?;

            // The IPFS cid of interest containing the row to fetch from IPFS from the left node that needs to be added as the last row of the keytable
            let cid_idx = self.ring.replication() - dist;
            let cid = leave_event.row_cid_of(cid_idx as usize);

            // recover keys not transferred from leaving node. These range from the block after the one that indentifies the keytable version and the current block number being processed (excluding events that still need to be processed, since these will be captured).
            debug_assert!(leave_event.key_table_at() < event_block_num);

            let raplay = ReplayBlocks::new(
                ReplayCause::new(leave_event.node(), dist),
                leave_event.key_table_at() + 1,
                (event_block_num, event_idx),
            );
            let replay_batch = self.replay_blocks(raplay).await?;
            let row_idx = self.ring.replication() as usize - 1;

            for keyed_event in replay_batch.iter() {
                self.update_table_from_event(row_idx, keyed_event)?;
            }

            let pin_events: Vec<PinningEvent> = replay_batch.into_iter().map(|e| e.pin).collect();
            let pin_batch = Batch::from(pin_events);

            return Ok(Some((cid, pin_batch)));
        }

        Ok(None)
    }
}

impl KeysDispatcher {
    pub async fn replay_blocks(&self, replay: ReplayBlocks) -> Result<Batch<KeyedPinningEvent>> {
        // Replay the events from the replay blocks
        let cause = replay.cause();
        let from_block = replay.from();
        let (to_block, break_event_idx) = replay.to();

        let mut pinning_batch = Batch::default();
        for block_num in from_block..=to_block {
            let block_hash = self.client.api().block_hash(block_num).await?;

            let batch = self
                .client
                .events_at(BlockInfo::new(block_num, block_hash))
                .await?;

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
                        pinning_batch.insert(keyed_event);
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
        let maybe_partition = self.ring.key_node_partition(key, self.client.node_id())?;

        if let Some(partition) = maybe_partition {
            if partition as u32 == self.ring.replication() {
                if dist_from_node == self.ring.replication() && key >= node {
                    return Ok(false);
                }
                return Ok(true);
            }
        }

        return Ok(false);
    }

    pub fn mutable_keytable(&mut self) -> &mut FaultTolerantKeyTable {
        &mut self.keytable
    }

    pub fn keytable(&self) -> &FaultTolerantKeyTable {
        &self.keytable
    }

    pub fn update_table_from_event(
        &mut self,
        row_idx: usize,
        keyed_event: &KeyedPinningEvent,
    ) -> Result<()> {
        match &keyed_event.pin {
            PinningEvent::Pin { cid } => {
                self.keytable
                    .insert(row_idx, keyed_event.key, cid.clone())?;
            }
            PinningEvent::RemovePin { .. } => {
                self.keytable.remove(row_idx, &keyed_event.key)?;
            }
            PinningEvent::UpdatePin { new_cid, .. } => {
                self.keytable
                    .insert(row_idx, keyed_event.key, new_cid.clone())?;
            }
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
        ReplayCause {
            node,
            dist_from_node,
        }
    }
}
