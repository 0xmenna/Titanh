use std::io::Cursor;

use ipfs_api_backend_hyper::IpfsApi;

mod controller;
mod db;
mod substrate;
mod types;
mod utils;
mod ipfs;

#[tokio::main]
async fn main() {
	// Example

	let client = ipfs_api_backend_hyper::IpfsClient::default();

	let res = client.pin_rm("QmdEJwJG1T9rzHvBD8i69HHuJaRgXRKEQCP7Bh1BVttZbU", true).await;

	match res {
		Ok(_) => println!("dajeeeeee"),
		Err(_) => println!("Shhiiiiit"),
	}
}
