use super::{
	chain::{
		titanh::{capsules::Event, runtime_types::titanh_runtime::RuntimeEvent},
		CapsuleKey, NodeId,
	},
	channels::PinningChannels,
	ipfs::Cid,
};
use crate::{
	controller::pinning_control::PinningNodeController, substrate::client::SubstrateClient,
};
use anyhow::Result;
use primitives::BlockNumber;
use std::{marker::PhantomData, sync::Arc};
use tokio::{
	sync::mpsc::{
		self, channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
	},
	task,
};

pub enum PinningEvent {
	Pin { cid: Cid },
	RemovePin { cid: Cid },
	UpdatePin { old_cid: Cid, new_cid: Cid },
}

pub struct PinningCapsuleEvent {
	/// The capsule key of the event
	pub key: CapsuleKey,
	/// The pinning specific event
	pub event: PinningEvent,
}

impl PinningCapsuleEvent {
	// Generates a pinning node event from a runtime event. If the runtime event is not an event of interest it returns `None`.
	pub fn try_from_runtime_event(event: RuntimeEvent) -> Option<Self> {
		let mut pinning_capsule_event = None;

		if let RuntimeEvent::Capsules(event) = event {
			// Capsule event
			match event {
				// Upload event
				Event::CapsuleUploaded { id, cid, .. } => {
					pinning_capsule_event =
						Some(Self { key: id, event: PinningEvent::Pin { cid: cid.0.to_vec() } })
				},
				// Update event
				Event::CapsuleContentChanged { capsule_id, old_cid, cid, .. } => {
					pinning_capsule_event = Some(Self {
						key: capsule_id,
						event: PinningEvent::UpdatePin {
							old_cid: old_cid.0.to_vec(),
							new_cid: cid.0.to_vec(),
						},
					})
				},
				// Deletion event
				Event::CapsuleDeleted { capsule_id, cid } => {
					pinning_capsule_event = Some(Self {
						key: capsule_id,
						event: PinningEvent::RemovePin { cid: cid.0.to_vec() },
					})
				},
				// ignore
				_ => {},
			}
		}

		pinning_capsule_event
	}
}

/// The pinning ring
pub struct PinningRing {
	ring: Vec<NodeId>,
	replication_factor: u32,
}

impl PinningRing {
	pub fn new(ring: Vec<NodeId>, replication_factor: u32) -> Self {
		Self { ring, replication_factor }
	}

	/// Looks for the closest node in the ring given a `target_key`
	fn binary_search_closest_node(&self, target_key: CapsuleKey) -> Result<usize> {
		if self.ring.is_empty() {
			return Err(anyhow::anyhow!("Pinning ring is empty"));
		}

		let mut low = 0;
		let mut high = self.ring.len() - 1;

		while low < high {
			let mid = (low + high) / 2;

			if self.ring[mid] == target_key {
				return Ok(mid);
			} else if self.ring[mid] < target_key {
				low = mid + 1;
			} else {
				high = mid;
			}
		}

		if self.ring[low] >= target_key {
			Ok(low)
		} else {
			Ok((low + 1) % self.ring.len())
		}
	}

	/// Returns true if the key must be handled by the input node in the ring.
	/// It can also return an error if the ring is empty
	pub fn is_key_owned_by_node(&self, key: CapsuleKey, node_id: NodeId) -> Result<bool> {
		// The closest node to `key`
		let next_node_idx = self.binary_search_closest_node(key)?;

		let ring_size = self.ring.len();
		let sum = next_node_idx + self.replication_factor as usize;

		let mut replica_nodes = Vec::new();
		if sum < self.ring.len() {
			replica_nodes.extend_from_slice(&self.ring[next_node_idx..=sum]);
		} else {
			let diff = sum - ring_size;
			replica_nodes.extend_from_slice(&self.ring[0..diff]);
			replica_nodes.extend_from_slice(&self.ring[next_node_idx..ring_size]);
		}

		let node_idx = replica_nodes.binary_search(&node_id);

		Ok(node_idx.is_ok())
	}
}

// Maybe it needs a channel rather than a vector of capsule events
pub struct PinningEventsPool {
	client_api: Arc<SubstrateClient>,
	/// Events to be processed before listening the channel of upcoming events
	events: Vec<PinningCapsuleEvent>,

	// Channels to receive and send a block number
	rx_block: Receiver<BlockNumber>,
	tx_block: Sender<BlockNumber>,
	// Channels to receive and send events
	rx_event: UnboundedReceiver<PinningCapsuleEvent>,
	tx_event: UnboundedSender<PinningCapsuleEvent>,
}

impl PinningEventsPool {
	pub fn new(client_api: Arc<SubstrateClient>) -> Self {
		// todo: gestire canali
		let channels = PinningChannels::new();
		let (tx_block, rx_block) = channel(1);
		let (tx_event, rx_event) = unbounded_channel();
		Self { client_api, events: Vec::new(), rx_block, tx_block, rx_event, tx_event }
	}

	pub fn add_events(&mut self, events: Vec<PinningCapsuleEvent>) {
		self.events.extend(events);
	}

	/// Pulls new finalized capsule events from the chain and produces them into a channel
	pub fn produce_capsule_events(&self) -> Result<()> {
		// Clone the Arc to use it in the thread that handles the event subscription
		let client_api = Arc::clone(&self.client_api);

		let subscription: task::JoinHandle<anyhow::Result<()>> = task::spawn(async move {
			let mut is_block_sent = false;
			let mut blocks_sub = client_api.get_api().blocks().subscribe_finalized().await?;

			while let Some(block) = blocks_sub.next().await {
				let block = block?;
				// TODO: remember to use u64 for blocknumber
				let events = client_api.pinning_events_at(block.hash().into());
			}

			Ok(())
		});

		// Main thread aspetta che gli viene comunicato il blocco
		// Legge il blocco dal canale, ha il riferimento del canale in lettura
		// Quando c'e il bloeiver<PinningCacco fa la get_events() per quelli vecchi leggendoli dal canale in lettura degli eventi
		// agiunge eventi a self => self.events.extend(events);
		// termina

		Ok(())
	}

	/// Consumes recieving events, first from the events `Vec` and then from the channel for new finalized events
	pub fn consume_capsule_events(&self) {
		// prima processa tutti gli eventi in self.events
		// legge dal canale e porcessa eventi
	}
}

// TODO: delete this is just a reference to how lifetime works
// Lifetime usage
// Il riferimento di ciao deve esistere finchè esiste la struct Ciao. Quinidi il ciclo di vita della variabile "ciao" è dipendente dalla struct.
pub struct Ciao<'a> {
	ciao: &'a str,
}

pub fn return_vector<'a>(x: &'a str, y: &'a str) -> &'a str {
	return x;
}
