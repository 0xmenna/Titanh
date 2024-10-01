use crate::types::cid::Cid;
use anyhow::Result;
use futures::TryStreamExt;
use ipfs_api_backend_hyper::Error as IpfsError;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient as ApiIpfsClient};
use rand::rngs::SmallRng as Randomness;
use rand::{Rng, SeedableRng};
use std::future::Future;

pub struct IpfsClient {
    /// The IPFS client replicas
    replicas: Vec<ApiIpfsClient>,
    /// The number of retries for pinning operations
    failure_retry: u8,
    /// The random number generator used for selecting a random replica
    rng: Randomness,
}

impl IpfsClient {
    pub fn new(replicas: Vec<ApiIpfsClient>, failure_retry: u8) -> Self {
        let rng = Randomness::from_entropy();
        Self {
            replicas,
            failure_retry,
            rng,
        }
    }

    pub async fn get(&mut self, cid: Cid) -> Result<Vec<u8>> {
        let client = self.select_client();
        let response = client
            .cat(cid.as_ref())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(|_| anyhow::anyhow!("error reading full file"))?;

        Ok(response)
    }

    // Add a pin
    pub async fn pin_add(&mut self, cid: &Cid) {
        self.pinning_op(cid, PinOp::Add).await
    }

    // Remove a pin
    pub async fn pin_remove(&mut self, cid: &Cid) {
        self.pinning_op(cid, PinOp::Remove).await
    }

    // Select a random client from the replicas.
    fn select_client(&mut self) -> &ApiIpfsClient {
        let idx = self.rng.gen_range(0..self.replicas.len());
        &self.replicas[idx]
    }

    async fn handle_pin_op<F, Fut, R>(op: F) -> Result<()>
    where
        // HRTB: The closure must work for any lifetime 'a
        F: Fn() -> Fut,
        // The future must not outlive 'a
        Fut: Future<Output = std::result::Result<R, IpfsError>>,
    {
        // Execute the closure with a reference to ApiIpfsClient
        match op().await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Check if the error is an API error with code 0
                if let IpfsError::Api(api_error) = &e {
                    if api_error.code == 0 {
                        // Ignore errors with code 0
                        return Ok(());
                    }
                }
                // Propagate other errors using anyhow::Result
                Err(e.into())
            }
        }
    }

    // Pinning operation. If the operation fails, retry it up to `failure_retry` times
    async fn pinning_op(&mut self, cid: &Cid, op: PinOp) {
        for _ in 0..self.failure_retry {
            let client = self.select_client();
            let response = match op {
                PinOp::Add => Self::handle_pin_op(|| client.pin_add(cid.as_ref(), true)).await,
                PinOp::Remove => Self::handle_pin_op(|| client.pin_rm(cid.as_ref(), true)).await,
            };

            if let Ok(_) = response {
                break;
            }
        }
    }
}

enum PinOp {
    Add,
    Remove,
}
