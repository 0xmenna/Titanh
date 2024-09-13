use std::str::FromStr;

use sp_core::H256;
use subxt::blocks::BlockRef;
use titanh_runtime::{AppRegistrar, Runtime};
use utils::SubstrateApiBuilder;

pub mod types;
pub mod utils;

use types::chain::{AppRegistrarEvents, RuntimeEvent};

#[tokio::main]
async fn main() {
	let api = SubstrateApiBuilder::default()
		.default_rpc_url()
		.keyring_material(
			"blush then predict bitter neutral later student true mom section echo gown",
			None,
		)
		.unwrap()
		.build()
		.await
		.unwrap();

	// Hex string with "0x" prefix
	let hex_str = "0x6c21f38c4c35590f91beaaad234a34576f7f888d439b6c0ea3b49dff74bc386a";

	// Remove the "0x" prefix and convert the hex string to H256
	let block_hash = H256::from_str(hex_str.trim_start_matches("0x")).expect("Invalid hex string");

	let events = api.api().events().at(BlockRef::from_hash(block_hash)).await.unwrap();
	for event in events.iter() {
		let event = event.unwrap();
		let pallet = event.pallet_name();
		let variant = event.variant_name();
		let field_values = event.field_values().unwrap();
		println!("{}", format!("{pallet}::{variant}: {field_values}"));
	}
}
