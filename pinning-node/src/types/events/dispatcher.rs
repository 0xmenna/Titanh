use super::NodeEvent;
use crate::{
	db::checkpointing::DbCheckpoint as DbClient,
	ipfs::client::IpfsClient,
	utils::{
		ref_builder::{MutableRef, Ref},
		traits::{Dispatcher, MutableDispatcher},
	},
};
use anyhow::Result;
use async_trait::async_trait;

/// Event dispatcher
pub type EventDispatcher = (Ref<DbClient>, MutableRef<IpfsClient>);

#[async_trait(?Send)]
impl MutableDispatcher<NodeEvent> for EventDispatcher {
	async fn dispatch(&mut self, event: &NodeEvent) -> Result<()> {
		match event {
			// Checkpointing event
			NodeEvent::BlockBarrier(checkpoint_event) => {
				self.0.dispatch(checkpoint_event).await?;
			},
			// Pinning event
			NodeEvent::Pinning { partition_num, keyed_event } => {
				let mut ipfs_client = self.1.borrow_mut();
				let dispatch_res = ipfs_client.dispatch(keyed_event).await;
				debug_assert!(dispatch_res.is_ok());
			},
			_ => {},
		};

		Ok(())
	}
}
