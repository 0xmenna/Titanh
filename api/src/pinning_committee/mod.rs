use crate::{titanh, TitanhApi};
use anyhow::Result;
use types::PinningRing;

pub struct PinningCommitteeApi<'a> {
	titanh: &'a TitanhApi,
}

impl<'a> From<&'a TitanhApi> for PinningCommitteeApi<'a> {
	fn from(titanh: &'a TitanhApi) -> Self {
		PinningCommitteeApi { titanh }
	}
}

impl PinningCommitteeApi<'_> {
	pub async fn pinning_ring(&self) -> Result<PinningRing> {
		let ring_state_query = titanh::storage().pinning_committee().pinning_nodes_ring();
		let hash_nodes_bounded = self.titanh.query(&ring_state_query, None).await?;
		let hash_nodes = hash_nodes_bounded.0.to_vec();
		let replication_factor_query =
			titanh::storage().pinning_committee().content_replication_factor();
		let replication_factor = self.titanh.query(&replication_factor_query, None).await?;
		let nodes_in_ring: PinningRing = PinningRing::new(hash_nodes, replication_factor);
		Ok(nodes_in_ring)
	}
}

pub mod types;
