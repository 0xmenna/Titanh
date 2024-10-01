use cli::Cli;
use controller::PinningNodeController;

mod cli;
mod controller;
mod db;
mod events;
mod ipfs;
mod substrate;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    let config = Cli::parse_config();

    let node = PinningNodeController::bootstrap(config).await;
    node.execute().await
}
