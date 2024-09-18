use super::NodeEvent;
use crate::{db::checkpointing::DbCheckpoint as DbClient, ipfs::client::IpfsClient};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Dispatcher<E> {
	async fn dispatch(&self, event: &E) -> Result<()>;
}

/// All dispatchers
pub type EventDispatcher<'a> = (&'a DbClient, &'a IpfsClient);

#[async_trait(?Send)]
impl<'a> Dispatcher<NodeEvent> for EventDispatcher<'a> {
	async fn dispatch(&self, event: &NodeEvent) -> Result<()> {
		match event {
			// Checkpointing event
			NodeEvent::BlockCheckpoint(checkpoint_event) => {
				self.0.dispatch(checkpoint_event).await?;
			},
			// Pinning event
			NodeEvent::Pinning(keyed_pinning_event) => {
				self.1.dispatch(keyed_pinning_event).await?;
			},
		};

		Ok(())
	}
}
