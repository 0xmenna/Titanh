use crate::{
    cli::Cli,
    db::checkpointing::DbCheckpoint,
    events::{consumer::NodeConsumer, dispatcher::NodeEventDispatcher, producer::NodeProducer},
    ipfs::client_builder::IpfsClientBuilder,
    substrate::client_builder::SubstrateClientBuilder,
    types::events_pool::NodeEventsPool,
};
use anyhow::Result;

pub struct PinningNodeController {
    /// Node event producer.
    /// A thread is spawned to pull events of real-time finalized blocks from the chain and produce them into the events pool. It only produces events relevant to the node.
    producer: NodeProducer,
    /// Node event consumer. It consumes and dispatches events from the events pool (that abstracts away a channel).
    consumer: NodeConsumer,
}

impl PinningNodeController {
    pub async fn bootstrap() -> Result<Self> {
        let config = Cli::parse_config();
        // Node checkpointing db
        let db = DbCheckpoint::from_config(&config);
        let checkpoint = db.get_checkpoint()?;
        // Block number until which the node has processed events and has an up to date keytable.
        log::info!("Checkpoint is at block number: {}", checkpoint.height());

        // Build the substrate client to read the blockchain related data
        let sub_client = SubstrateClientBuilder::from_config(&config, &db)
            .build()
            .await?;
        log::info!(
            "Substrate client initialized at block number: {}, with ID: {}",
            sub_client.height(),
            hex::encode(sub_client.node_id())
        );
        let ring = sub_client.ring().await;

        let sub_client = sub_client.arc();
        let events_pool = NodeEventsPool::new().mutable_ref();

        let start_block_recovering = checkpoint.height() + 1;
        let producer = NodeProducer::new(
            sub_client.clone(),
            events_pool.clone(),
            start_block_recovering,
            ring.height(),
            config.latency,
        );

        // Build the IPFS client for ipfs related operations (e.g. pinning, unpinning, reading files)
        let ipfs_client = IpfsClientBuilder::from_config(&config, checkpoint.pin_counts())
            .build()
            .await?;
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
            checkpoint.height(),
            checkpoint.keytable(),
        );
        let consumer = NodeConsumer::new(events_pool, dispatcher);

        Ok(Self { producer, consumer })
    }

    pub async fn execute(mut self) -> Result<()> {
        // Spawn the producer thread
        let producer_handle = self.producer.produce_events();

        // Run the consumer task concurrently
        self.consumer.consume_events().await?;

        // Wait for the producer task to complete
        producer_handle.await?
    }
}
