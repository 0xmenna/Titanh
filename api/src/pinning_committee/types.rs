use crate::{
    capsules_types::CapsuleKey,
    common_types::{BlockHash, BlockInfo},
};
use anyhow::Result;
use sp_core::H256;

/// A pinning node's identifier in the ring
pub type NodeId = H256;

/// The pinning ring
pub struct PinningRing {
    ring: Vec<NodeId>,
    replication_factor: u32,
    /// The block info at which the ring was first initialized
    block: BlockInfo,
}

pub enum NodeLookup {
    /// The node is already in the ring, at the given index
    Found(usize),
    /// The node is not in the ring, but it should be inserted at the given index
    NotFound(usize),
}

impl PinningRing {
    pub fn new(ring: Vec<NodeId>, replication_factor: u32, block: BlockInfo) -> Self {
        Self {
            ring,
            replication_factor,
            block,
        }
    }

    fn node_lookup(&self, node_id: &NodeId) -> NodeLookup {
        let idx = self.ring.binary_search(node_id);

        match idx {
            Ok(node_idx) => NodeLookup::Found(node_idx),
            Err(insert_idx) => NodeLookup::NotFound(insert_idx),
        }
    }

    /// Insert a node in the ring and return the index where it was inserted
    pub fn insert_node(&mut self, node_id: &NodeId) -> Result<usize> {
        // Find the position where the node should be inserted
        let lookup = self.node_lookup(node_id);

        // The node should not already be in the ring
        if let NodeLookup::NotFound(idx) = lookup {
            self.ring.insert(idx, *node_id);

            Ok(idx)
        } else {
            Err(anyhow::anyhow!("Node should not already be in the ring"))
        }
    }

    /// Removes a node from the ring and retrun the update
    pub fn remove_node(&mut self, node_id: &NodeId) -> Result<usize> {
        let idx = self.node(node_id)?;
        self.ring.remove(idx);

        Ok(idx)
    }

    /// Returns the index of the node in the ring, if it exists. Else, it returns an error
    pub fn node(&self, node: &NodeId) -> Result<usize> {
        let lookup = self.node_lookup(node);
        match lookup {
            NodeLookup::Found(idx) => Ok(idx),
            NodeLookup::NotFound(_) => Err(anyhow::anyhow!("Node not found")),
        }
    }

    /// Returns the distance of node_id relative to node at `idx` in the ring considering clockwise direction.
    pub fn distance_from_idx(&self, idx: usize, node_id: &NodeId) -> Result<u32> {
        let node_idx = self.node(node_id)?;
        let total_nodes = self.len();

        if idx >= node_idx {
            Ok((idx - node_idx) as u32)
        } else {
            Ok((total_nodes - idx + node_idx) as u32)
        }
    }

    /// Returns the distance of `node_a` relative to `node_b` in the ring considering clockwise direction.
    pub fn distance_between(&self, node_a: &NodeId, node_b: &NodeId) -> Result<u32> {
        let idx_a = self.node(node_a)?;
        let idx_b = self.node(node_b)?;
        let total_nodes = self.len();

        if idx_b >= idx_a {
            Ok((idx_b - idx_a) as u32)
        } else {
            Ok((total_nodes - idx_b + idx_a) as u32)
        }
    }

    /// Looks for the closest node in the ring given a `target_key`
    fn binary_search_closest_node(&self, target_key: CapsuleKey) -> Result<usize> {
        if self.ring.is_empty() {
            return Err(anyhow::anyhow!("Pinning ring is empty"));
        }

        let mut low = 0;
        let mut high = self.ring.len() - 1;

        while low < high {
            let mid = (low + high) / 2;

            if self.ring[mid] == target_key {
                return Ok(mid);
            } else if self.ring[mid] < target_key {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if self.ring[low] >= target_key {
            Ok(low)
        } else {
            Ok((low + 1) % self.ring.len())
        }
    }

    /// Returns the index of the partition to which the key belongs, if the key must be handled by the input node in the ring.
    pub fn key_node_partition(&self, key: CapsuleKey, node_id: NodeId) -> Result<Option<usize>> {
        // The closest node to `key`
        let next_node_idx = self.binary_search_closest_node(key)?;

        let ring_size = self.ring.len();
        let sum = next_node_idx + self.replication_factor as usize;

        let mut replica_nodes = Vec::new();
        if sum < self.ring.len() {
            replica_nodes.extend_from_slice(&self.ring[next_node_idx..sum]);
        } else {
            let diff = sum - ring_size;
            replica_nodes.extend_from_slice(&self.ring[next_node_idx..ring_size]);
            replica_nodes.extend_from_slice(&self.ring[0..diff]);
        }

        let partition_idx = replica_nodes.iter().position(|&node| node == node_id);
        Ok(partition_idx)
    }

    /// Get the node at the given index, it panics if the index is out of bounds
    pub fn get(&self, idx: usize) -> &NodeId {
        self.ring.get(idx).unwrap()
    }

    pub fn len(&self) -> usize {
        self.ring.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    pub fn replication(&self) -> u32 {
        self.replication_factor
    }

    pub fn at(&self) -> BlockHash {
        self.block.hash
    }

    pub fn height(&self) -> u32 {
        self.block.number
    }
}
