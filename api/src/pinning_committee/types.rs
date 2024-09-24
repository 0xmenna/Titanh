use crate::capsules_types::CapsuleKey;
use anyhow::Result;
use sp_core::H256;

/// A pinning node's identifier in the ring
pub type NodeId = H256;

/// The pinning ring
pub struct PinningRing {
	ring: Vec<NodeId>,
	replication_factor: u32,
}

pub struct UpdateRing {
	pub nodes: [NodeId; 2],
	pub ranges: [(CapsuleKey, CapsuleKey); 2],
}

impl UpdateRing {
	pub fn node_range(&self, node_id: &NodeId) -> Option<(CapsuleKey, CapsuleKey)> {
		if self.nodes[0] == *node_id {
			Some(self.ranges[0])
		} else if self.nodes[1] == *node_id {
			Some(self.ranges[1])
		} else {
			None
		}
	}
}

pub enum NodeLookup {
	/// The node is already in the ring, at the given index
	Found(usize),
	/// The node is not in the ring, but it should be inserted at the given index
	NotFound(usize),
}

impl PinningRing {
	pub fn new(ring: Vec<NodeId>, replication_factor: u32) -> Self {
		Self { ring, replication_factor }
	}

	fn node_lookup(&self, node_id: NodeId) -> NodeLookup {
		let idx = self.ring.binary_search(&node_id);

		match idx {
			Ok(node_idx) => NodeLookup::Found(node_idx),
			Err(insert_idx) => NodeLookup::NotFound(insert_idx),
		}
	}

	/// Insert a node in the ring and return the update
	pub fn insert_node(&mut self, node_id: NodeId) -> Result<UpdateRing> {
		// Find the position where the node should be inserted
		let lookup = self.node_lookup(node_id);

		// The node should not already be in the ring
		if let NodeLookup::NotFound(idx) = lookup {
			self.ring.insert(idx, node_id);
			let update = self.update_info(idx);

			Ok(update)
		} else {
			Err(anyhow::anyhow!("Node should not already be in the ring"))
		}
	}

	/// Removes a node from the ring and retrun the update
	pub fn remove_node(&mut self, node_id: NodeId) -> Result<UpdateRing> {
		let lookup = self.node_lookup(node_id);

		// The node should already be in the ring
		if let NodeLookup::Found(idx) = lookup {
			let update = self.update_info(idx);
			self.ring.remove(idx);

			Ok(update)
		} else {
			Err(anyhow::anyhow!("Node should be in the ring"))
		}
	}

	fn update_info(&self, idx: usize) -> UpdateRing {
		let k = self.replication_factor as usize;

		// First node to be impacted
		let a_idx = self.add_idx(idx, k);
		let node_a = self.ring[a_idx];
		// Impacted range
		let prev_idx = self.sub_idx(idx, 1);
		let range_a = (self.ring[prev_idx], self.ring[idx]);

		// Second node to be impacted
		let b_idx = self.add_idx(idx, 1);
		let node_b = self.ring[b_idx];
		// Impacted range
		let prev_idx = self.sub_idx(idx, k);
		let next_idx = self.add_idx(prev_idx, 1);
		let range_b = (self.ring[prev_idx], self.ring[next_idx]);

		UpdateRing { nodes: [node_a, node_b], ranges: [range_a, range_b] }
	}

	fn sub_idx(&self, idx: usize, value: usize) -> usize {
		let ring_size = self.ring.len();
		if idx < value {
			(idx + self.len() - value) % ring_size
		} else {
			idx - value
		}
	}

	fn add_idx(&self, idx: usize, value: usize) -> usize {
		(idx + value) % self.ring.len()
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

	/// Returns the number of the partition to which the key belongs, if the key must be handled by the input node in the ring.
	/// It can also return an error if the ring is empty
	pub fn key_node_partition(&self, key: CapsuleKey, node_id: NodeId) -> Result<Option<usize>> {
		// The closest node to `key`
		let next_node_idx = self.binary_search_closest_node(key)?;

		let ring_size = self.ring.len();
		let sum = next_node_idx + self.replication_factor as usize;

		let mut replica_nodes = Vec::new();
		if sum < self.ring.len() {
			replica_nodes.extend_from_slice(&self.ring[next_node_idx..=sum]);
		} else {
			let diff = sum - ring_size;
			replica_nodes.extend_from_slice(&self.ring[0..diff]);
			replica_nodes.extend_from_slice(&self.ring[next_node_idx..ring_size]);
		}

		let partition_idx = replica_nodes.binary_search(&node_id);

		match partition_idx {
			Ok(idx) => Ok(Some(idx)),
			Err(_) => Ok(None),
		}
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
}
