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

impl PinningRing {
	pub fn new(ring: Vec<NodeId>, replication_factor: u32) -> Self {
		Self { ring, replication_factor }
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

	/// Returns true if the key must be handled by the input node in the ring.
	/// It can also return an error if the ring is empty
	pub fn is_key_owned_by_node(&self, key: CapsuleKey, node_id: NodeId) -> Result<bool> {
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

		let node_idx = replica_nodes.binary_search(&node_id);

		Ok(node_idx.is_ok())
	}
}
