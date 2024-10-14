use anyhow::Result;
use garbage_consumer::GarbageCollectorConsumer;

mod garbage_consumer;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let (collector_seed, rpc_url, key_range) = utils::load_env()?;

    // Create a new garbage collector consumer that consumes destroying events from the chain
    let garbage_collector = GarbageCollectorConsumer::new(collector_seed, rpc_url, key_range);

    // Start the garbage collector
    garbage_collector.start().await
}
