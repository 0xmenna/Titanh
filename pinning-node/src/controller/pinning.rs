use crate::{
	db::checkpointing::DbCheckpoint,
	ipfs::{client::IpfsClient, client_builder::IpfsClientBuilder},
	substrate::{client2::SubstratePinningClient, client_builder::SubstrateClientBuilder},
	types::events::events_pool::NodeEventsPool,
	utils::{
		config::Config,
		ref_builder::{self, AtomicRef, MutableRef, Ref},
		traits::ClientBuilder,
	},
};

pub struct PinningNodeController {
	/// The IPFS client
	ipfs: MutableRef<IpfsClient>,
	/// The substrate client for chain queries
	substrate_client: AtomicRef<SubstratePinningClient>,
	/// The checkpointing db
	db: Ref<DbCheckpoint>,
	/// The events pool
	events_pool: NodeEventsPool,
}

impl PinningNodeController {
	pub async fn bootstrap() -> Self {
		let config = Config::from_json();

		// Build the ipfs client
		let ipfs = IpfsClientBuilder::from_config(&config).build().await;
		let ipfs = ref_builder::create_mutable_ref(ipfs);

		// Build the substrate pinning client for interacting with the chain
		let substrate_client = SubstrateClientBuilder::from_config(&config).build().await;
		let substrate_client = ref_builder::create_atomic_ref(substrate_client);

		// Create the checkpointing db
		let db = ref_builder::create_ref(DbCheckpoint::new());

		// Create the events pool to manage chain events
		let events_pool = NodeEventsPool::new(substrate_client.clone(), db.clone(), ipfs.clone());

		Self { ipfs, substrate_client, db, events_pool }
	}

	pub async fn execute(mut self) {
		let event_producing_handle = self.events_pool.produce_events().await.unwrap();

		self.events_pool.consume_events().await.unwrap();

		// Wait because node continues to handle events in the background
		let _ = event_producing_handle.await.unwrap();
	}
}
