use controller::pinning::PinningNodeController;

mod controller;
mod db;
mod ipfs;
mod substrate;
mod types;
mod utils;

#[tokio::main]
async fn main() {
	PinningNodeController::bootstrap().await.execute().await
}
