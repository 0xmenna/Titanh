use crate::{
    db::checkpointing::DbCheckpoint,
    events::{consumer::NodeConsumer, dispatcher::NodeEventDispatcher, producer::NodeProducer},
    ipfs::client_builder::IpfsClientBuilder,
    substrate::client_builder::SubstrateClientBuilder,
    types::events_pool::NodeEventsPool,
    utils::{config::Config, traits::ClientBuilder},
};

pub struct PinningNodeController {
    /// Node event producer. It pulls events from the chain and produces them into the events pool.
    /// A thread is spawned to handle finalized blocks in real-time. Blocks contain events of interest to the node.
    producer: NodeProducer,
    /// Node event consumer. It consumes and dispatches events from the events pool, that abstracts away a channel.
    consumer: NodeConsumer,
}

impl PinningNodeController {
    pub async fn bootstrap(config: Config) -> Self {
        // Build the substrate client to read the blockchain related data
        let sub_client = SubstrateClientBuilder::from_config(&config).build().await;
        log::info!(
            "Substrate client initialized at block: {:?}",
            sub_client.block_num()
        );

        let ring = sub_client.ring().await;
        let replication_factor = ring.replication();

        // Node checkpointing db
        let db = DbCheckpoint::from_config(replication_factor, config.node_id());
        let checkpoint = db.get_checkpoint().unwrap();
        // Block number until which the node has processed events.
        // The keytable is updated at this block.
        let block_num = checkpoint.at();

        let sub_client = sub_client.arc();
        let events_pool = NodeEventsPool::new().mutable_ref();

        let producer = NodeProducer::new(sub_client.clone(), events_pool.clone(), block_num);

        // Build the IPFS client for ipfs related operations (e.g. pinning, unpinning, reading files)
        let ipfs_client = IpfsClientBuilder::from_config(&config).build().await;

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
