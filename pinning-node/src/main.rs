use controller::PinningNodeController;

mod controller;
mod db;
mod events;
mod ipfs;
mod substrate;
mod types;
mod utils;

#[tokio::main]
async fn main() {
	let node = PinningNodeController::bootstrap().await;
	node.execute().await
}
