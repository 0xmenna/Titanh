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
    /// The random number generator used for selecting a random client
    rng: Randomness,
    /// Pinning metadata of the client
    pinning_metadata: PinMetadata,
}

impl IpfsClient {
    pub fn new(
        ipfs_clients: Vec<ApiIpfsClient>,
        failure_retry: u8,
        pin_counts: Vec<(Cid, u32)>,
    ) -> Self {
        let rng = Randomness::from_entropy();

        let mut pin_counts_map = HashMap::new();
        for (cid, count) in pin_counts {
            pin_counts_map.insert(cid, (count, None));
        }
        let pinning_metadata = PinMetadata::new(pin_counts_map);

        Self {
            clients: ipfs_clients,
            failure_retry,
            rng,
            pinning_metadata,
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
        self.pinning_op(cid, PinOp::Add).await.unwrap();
    }

    // Remove a pin
    pub async fn pin_remove(&mut self, cid: &Cid) -> Result<()> {
        self.pinning_op(cid, PinOp::Remove).await
    }

    // Select a random ipfs client from the available nodes.
    fn select_client(&mut self) -> &ApiIpfsClient {
        let idx = self.rng.gen_range(0..self.clients.len());
        let node = &self.clients[idx];
        node
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
                // If the cid is already pinned, increment the pin count
                if self.pinning_metadata.pin_exists(cid) {
                    self.pinning_metadata.add_cid_pinning_ref(cid)?;
                    return Ok(());
                }

                for _ in 0..self.failure_retry {
                    let client = self.select_client();
                    let res = Self::handle_pin_op(|| client.pin_add(cid.as_ref(), true)).await;

                    if res.is_ok() {
                        self.pinning_metadata.insert_cid_pinning_ref(cid.clone());
                        break;
                    }
                }
            }
            PinOp::Remove => {
                let remaining_pins = self.pinning_metadata.decrement_cid_pinning_ref(cid)?;

                if remaining_pins == 0 {
                    for client in self.clients.iter() {
                        // If the client is offline and is not able to remove the pin, ignore the error
                        let res = Self::handle_pin_op(|| client.pin_rm(cid.as_ref(), true)).await;
                        if res.is_ok() {
                            update_cid_pins_to_flush(
                                &mut self.pinning_metadata.pins_to_flush,
                                cid,
                                0,
                                &mut None,
                            );
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn flush_pins(&mut self) -> Vec<(Cid, u32)> {
        self.pinning_metadata.flush_pins()
    }
}

#[derive(PartialEq, Eq)]
enum PinOp {
    Add,
    Remove,
}

struct PinMetadata {
    /// The number of pins for a given cid and whether it must be flushed (by specifying the cid position in the below vector)
    pin_counts: HashMap<Cid, (u32, Option<usize>)>,
    /// Pins to flush at the end of a batch
    pins_to_flush: Vec<(Cid, u32)>,
}

impl PinMetadata {
    fn new(pin_counts: HashMap<Cid, (u32, Option<usize>)>) -> Self {
        Self {
            pin_counts,
            pins_to_flush: Vec::new(),
        }
    }

    fn pin_exists(&self, cid: &Cid) -> bool {
        self.pin_counts.contains_key(cid)
    }

    fn decrement_cid_pinning_ref(&mut self, cid: &Cid) -> Result<u32> {
        let (count, idx_option) = self.pin_counts.get_mut(cid).ok_or(anyhow::anyhow!(
            "Cannot remove pin for cid: {:?} because it does not exist",
            cid
        ))?;

        *count -= 1;

        if *count > 0 {
            update_cid_pins_to_flush(&mut self.pins_to_flush, cid, *count, idx_option);
        }

        Ok(*count)
    }

    fn insert_cid_pinning_ref(&mut self, cid: Cid) {
        let mut idx = None;
        let count = 1;
        update_cid_pins_to_flush(&mut self.pins_to_flush, &cid, count, &mut idx);
        self.pin_counts.insert(cid, (count, idx));
    }

    fn add_cid_pinning_ref(&mut self, cid: &Cid) -> Result<()> {
        let (count, idx_option) = self.pin_counts.get_mut(cid).ok_or(anyhow::anyhow!(
            "Cannot add pin for cid: {:?} because it is not pinned",
            cid
        ))?;

        *count += 1;

        update_cid_pins_to_flush(&mut self.pins_to_flush, cid, *count, idx_option);

        Ok(())
    }

    fn flush_pins(&mut self) -> Vec<(Cid, u32)> {
        let pins_to_flush = std::mem::take(&mut self.pins_to_flush);

        pins_to_flush
    }
}

fn update_cid_pins_to_flush(
    pins_to_flush: &mut Vec<(Cid, u32)>,
    cid: &Cid,
    count: u32,
    idx_option: &mut Option<usize>,
) {
    match idx_option {
        Some(idx) => {
            pins_to_flush[*idx] = (cid.clone(), count);
        }
        None => {
            let idx = pins_to_flush.len();
            pins_to_flush.push((cid.clone(), count));
            *idx_option = Some(idx);
        }
    }
}
