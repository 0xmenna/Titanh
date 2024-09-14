use sp_core::H256;
use std::str::FromStr;
use subxt::blocks::BlockRef;

mod substrate;
mod types;
mod utils;

use substrate::{client::titanh, client_builder::SubstrateClientBuilder};

#[tokio::main]
async fn main() {
	// Example
	let client = SubstrateClientBuilder::new()
		.keyring_material(
			"also arena hammer relief judge vintage rather intact elder review until filter",
			None,
		)
		.unwrap()
		.build()
		.await
		.unwrap();

	let app_registrar_query = titanh::storage().app_registrar().current_app_id();

	let id = client.query(&app_registrar_query, None).await.unwrap();

	println!("Current app id: {:?}", id);
}
