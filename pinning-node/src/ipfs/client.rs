use crate::types::events::dispatcher::Dispatcher;
use crate::types::events::{KeyedPinningEvent, PinningEvent};
use crate::types::ipfs::Cid;
use anyhow::Result;
use async_trait::async_trait;
use ipfs_api_backend_hyper::Error as IpfsError;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient as ApiIpfsClient};

pub struct IpfsClient(pub ApiIpfsClient);

impl IpfsClient {
	pub fn new(client: ApiIpfsClient) -> Self {
		Self(client)
	}

	pub async fn try_pin_add(&self, cid: &Cid) -> Result<()> {
		if self
			.0
			.pin_add(cid.as_ref(), true)
			.await
			.is_err_and(|maybe_err| self.is_ipfs_err(maybe_err))
		{
			return Err(anyhow::anyhow!("Ipfs pin add failed"));
		}

		Ok(())
	}

	pub async fn try_pin_rm(&self, cid: &Cid) -> Result<()> {
		if self
			.0
			.pin_rm(cid.as_ref(), true)
			.await
			.is_err_and(|maybe_err| self.is_ipfs_err(maybe_err))
		{
			return Err(anyhow::anyhow!("IPFS pin remove failed"));
		}

		Ok(())
	}

	fn is_ipfs_err(&self, maybe_err: IpfsError) -> bool {
		matches!(maybe_err, IpfsError::Api(error) if error.code != 0)
	}
}

#[async_trait(?Send)]
impl Dispatcher<KeyedPinningEvent> for IpfsClient {
	async fn dispatch(&self, keyed_event: &KeyedPinningEvent) -> Result<()> {
		match &keyed_event.event {
			PinningEvent::Pin { cid } => {
				self.try_pin_add(cid).await?;
			},

			PinningEvent::UpdatePin { old_cid, new_cid } => {
				self.try_pin_rm(old_cid).await?;
				self.try_pin_add(new_cid).await?;
			},

			PinningEvent::RemovePin { cid } => {
				self.try_pin_rm(cid).await?;
			},
		};

		Ok(())
	}
}
