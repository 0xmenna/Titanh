use anyhow::Result;
use pinning::PinningNodeController;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    // Bootstrap the node
    let node = PinningNodeController::bootstrap().await?;
    // Execute the node
    node.execute().await
}
