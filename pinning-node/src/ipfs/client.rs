use crate::db::checkpointing::DbCheckpoint;
use crate::substrate::client::SubstratePinningClient;
use crate::types::cid::Cid;
use crate::types::events::{KeyedPinningEvent, PinningEvent, RingUpdateEvent};
use crate::types::keytable::FaultTolerantBTreeMap;
use crate::utils::ref_builder::{MutableRef, Ref};
use crate::utils::traits::MutableDispatcher;
use anyhow::Result;
use api::pinning_committee_types::{NodeId, PinningRing};
use async_trait::async_trait;
use codec::Decode;
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
	/// The substrate pinning client api => TODO: change name
	api: Ref<SubstratePinningClient>,
	/// The checkpointing db
	db: Ref<DbCheckpoint>,
	/// A mutable reference to the pinning node ring
	ring: MutableRef<PinningRing>,
	/// A mutable reference to the key map managed by the pinning node
	key_map: MutableRef<FaultTolerantBTreeMap>,
	/// The node id bounded to the client
	node_id: NodeId,
}

impl IpfsClient {
	pub fn new(
		replicas: Vec<ApiIpfsClient>,
		failure_retry: u8,
		ring: MutableRef<PinningRing>,
		key_map: MutableRef<FaultTolerantBTreeMap>,
		node_id: NodeId,
	) -> Self {
		let rng = Randomness::from_entropy();
		Self { replicas, failure_retry, rng, ring, key_map, node_id }
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
			},
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

#[async_trait(?Send)]
impl MutableDispatcher<KeyedPinningEvent> for IpfsClient {
	async fn dispatch(&mut self, keyed_event: &KeyedPinningEvent) -> Result<()> {
		match &keyed_event.event {
			PinningEvent::Pin { cid } => {
				self.pin_add(cid).await;
				self.key_map.borrow_mut().insert(keyed_event.key, cid);
			},

			PinningEvent::UpdatePin { old_cid, new_cid } => {
				self.pin_remove(old_cid).await;
				self.pin_add(new_cid).await;
				self.key_map.borrow_mut().insert(keyed_event.key, new_cid);
			},

			PinningEvent::RemovePin { cid } => {
				self.pin_remove(cid).await;
				self.key_map.borrow_mut().remove(&keyed_event.key);
			},
		};

		Ok(())
	}
}

#[async_trait(?Send)]
impl MutableDispatcher<RingUpdateEvent> for IpfsClient {
	async fn dispatch(&mut self, event: &RingUpdateEvent) -> Result<()> {
		match event {
			RingUpdateEvent::NewPinningNode(node_id) => {
				let update = self.ring.borrow_mut().insert_node(node_id.clone())?;

				let key_range = update.node_range(&self.node_id);

				if let Some(range) = key_range {
					let key_values = self.key_map.borrow().range(&range.0, &range.1);
					for (key, cid) in key_values {
						self.pin_remove(&cid).await;
						self.key_map.borrow_mut().remove(&key);
					}
				}
			},
			RingUpdateEvent::RemovePinningNode { node_id, db_keys } => {
				let update = self.ring.borrow_mut().remove_node(node_id.clone())?;
				let key_range = update.node_range(&self.node_id);

				if let Some(_) = key_range {
					// Decode the db_keys and add the keys to the key map
					let transferrred_map = FaultTolerantBTreeMap::decode(&mut db_keys.as_ref())?;
					self.key_map.borrow_mut().merge(&transferrred_map);

					// recover past events of the keys within the transferred map
					let block_checkpoint = self.db.
					let block_num = transferrred_map.at();


				}
			},
		};

		Ok(())
	}
}
