use crate::types::cid::Cid;
use anyhow::Result;
use futures::TryStreamExt;
use ipfs_api_backend_hyper::Error as IpfsError;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient as ApiIpfsClient};
use rand::rngs::SmallRng as Randomness;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::future::Future;

pub struct IpfsClient {
    /// The IPFS clients
    clients: Vec<ApiIpfsClient>,
    /// The number of retries for pinning operations
    failure_retry: u8,
    /// The random number generator used for selecting a random replica
    rng: Randomness,
    /// The ipfs client idx that is currently pinning a cid
    pinning_client: HashMap<Cid, usize>,
}

impl IpfsClient {
    pub fn new(ipfs_clients: Vec<ApiIpfsClient>, failure_retry: u8) -> Self {
        let rng = Randomness::from_entropy();
        Self {
            clients: ipfs_clients,
            failure_retry,
            rng,
            pinning_client: HashMap::new(),
        }
    }

    pub async fn get(&mut self, cid: Cid) -> Result<Vec<u8>> {
        let (_, client) = self.select_client();
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
        self.pinning_op(cid, PinOp::Add).await.unwrap();
    }

    // Remove a pin
    pub async fn pin_remove(&mut self, cid: &Cid) -> Result<()> {
        self.pinning_op(cid, PinOp::Remove).await
    }

    // Select a random ipfs client from the available nodes.
    fn select_client(&mut self) -> (usize, &ApiIpfsClient) {
        let idx = self.rng.gen_range(0..self.clients.len());
        let node = &self.clients[idx];
        (idx, node)
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
    async fn pinning_op(&mut self, cid: &Cid, op: PinOp) -> Result<()> {
        match op {
            PinOp::Add => {
                for _ in 0..self.failure_retry {
                    let (client_idx, client) = self.select_client();
                    let res = Self::handle_pin_op(|| client.pin_add(cid.as_ref(), true)).await;

                    if res.is_ok() {
                        self.pinning_client.insert(cid.clone(), client_idx);
                        break;
                    }
                }
            }
            PinOp::Remove => {
                let client_idx = self.pinning_client.get(&cid);
                if let Some(client_idx) = client_idx {
                    let client = &self.clients[*client_idx];
                    // If the client is offline and is not able to remove the pin, ignore the error
                    let _ = Self::handle_pin_op(|| client.pin_rm(cid.as_ref(), true)).await;
                } else {
                    return Err(anyhow::anyhow!("No client is pinning the cid"));
                }
            }
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq)]
enum PinOp {
    Add,
    Remove,
}
