use crate::{
	common_types::BlockNumber,
	titanh::{self},
	TitanhApi,
};
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
	pub async fn pinning_ring(&self, block_num: Option<BlockNumber>) -> Result<PinningRing> {
		let (block_hash, block_num) = if let Some(num) = block_num {
			let block_hash = self.titanh.block_hash(num).await?;
			(block_hash, num)
		} else {
			let block = self.titanh.current_block().await?;
			(block.hash, block.number)
		};

		let ring_query = titanh::storage().pinning_committee().pinning_nodes_ring();
		let ring = self.titanh.query(&ring_query, Some(block_hash.clone())).await?;
		let ring = ring.0.to_vec();

		let replication_factor_query =
			titanh::storage().pinning_committee().content_replication_factor();
		let replication_factor =
			self.titanh.query(&replication_factor_query, Some(block_hash)).await?;

		let pinning_ring = PinningRing::new(ring, replication_factor, block_num);
		Ok(pinning_ring)
	}
}

pub mod types;
