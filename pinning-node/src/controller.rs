use crate::{
    cli::Cli,
    db::checkpointing::DbCheckpoint,
    events::{consumer::NodeConsumer, dispatcher::NodeEventDispatcher, producer::NodeProducer},
    ipfs::client_builder::IpfsClientBuilder,
    substrate::client_builder::SubstrateClientBuilder,
    types::events_pool::NodeEventsPool,
    utils::traits::ClientBuilder,
};

pub struct PinningNodeController {
    /// Node event producer.
    /// A thread is spawned to pull events of real-time finalized blocks from the chain and produce them into the events pool. It only produces events relevant to the node.
    producer: NodeProducer,
    /// Node event consumer. It consumes and dispatches events from the events pool (that abstracts away a channel).
    consumer: NodeConsumer,
}

impl PinningNodeController {
    pub async fn bootstrap() -> Self {
        let config = Cli::parse_config();
        // Build the substrate client to read the blockchain related data
        let sub_client = SubstrateClientBuilder::from_config(&config).build().await;
        log::info!(
            "Substrate client initialized at block number: {}, with ID: {}",
            sub_client.height(),
            hex::encode(sub_client.node_id())
        );

        let ring = sub_client.ring().await;
        let replication_factor = ring.replication();

        // Node checkpointing db
        let db = DbCheckpoint::from_config(replication_factor, config.node_id(), config.idx);
        let checkpoint = db.get_checkpoint().unwrap();
        log::info!("Checkpoint is at block number: {}", checkpoint.height());
        // Block number until which the node has processed events.
        // The keytable is updated at this block number.
        let block_num = checkpoint.height();

        let sub_client = sub_client.arc();
        let events_pool = NodeEventsPool::new().mutable_ref();

        let producer = NodeProducer::new(sub_client.clone(), events_pool.clone(), block_num);

        // Build the IPFS client for ipfs related operations (e.g. pinning, unpinning, reading files)
        let ipfs_client = IpfsClientBuilder::from_config(&config).build().await;
        log::info!(
            "IPFS client initialized successfully using replicas: {:?}",
            config.ipfs_peers
        );
        // Event dispatcher
        let dispatcher = NodeEventDispatcher::from_config(
            db,
            ipfs_client,
            sub_client,
            ring,
            checkpoint.keytable(),
            block_num,
        );
        let consumer = NodeConsumer::new(events_pool, dispatcher);

        Self { producer, consumer }
    }

    pub async fn execute(mut self) {
        let producer_handle = self.producer.produce_events().await.unwrap();

        self.consumer.consume_events().await.unwrap();

        // Wait because node continues to handle events in the background
        let _ = producer_handle.await.unwrap();
    }
}
