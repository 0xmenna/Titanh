use anyhow::Result;
use ipfs_api_backend_hyper::Error::Api;
use ipfs_api_backend_hyper::{ApiError, IpfsApi, IpfsClient};
pub struct IpfsClientWrapper(IpfsClient);
//TODO: duplicated code needs to be refactored
impl IpfsClientWrapper {
	pub fn new(client: IpfsClient) -> Self {
		IpfsClientWrapper(client)
	}

	pub async fn try_pin_add(&self, cid: String) -> Result<()> {
		if let Err(e) = self.0.pin_add(&cid, true).await {
			match e {
				Api(error) => {
					if error.code != 0 {
						Err(Api(error).into())
					} else {
						Ok(())
					}
				},
				_ => Err(e.into()),
			}
		} else {
			Ok(())
		}
	}

	pub async fn try_pin_rm(&self, cid: String) -> Result<()> {
		if let Err(e) = self.0.pin_rm(&cid, true).await {
			match e {
				Api(error) => {
					if error.code != 0 {
						Err(Api(error).into())
					} else {
						Ok(())
					}
				},
				_ => Err(e.into()),
			}
		} else {
			Ok(())
		}
	}
}
