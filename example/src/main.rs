use anyhow::Result;
use titan_api::TitanhApiBuilder;
use types::Certificate;
use utils::Config;

mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
	// read app configuration
	let config = Config::from_json();
	// build the Titanh API
	let api = TitanhApiBuilder::rpc(&config.chain_rpc).seed(&config.seed).build().await;
	// get the app registrar api to create the app
	let app_registrar = api.app_registrar();

	println!("Creating app...");
	let (app, _) = app_registrar.create_app().await?;

	// get the capsules api and configure it with the IPFS RPC URL and the app id
	let capsules = api.capsules().config(&config.ipfs_rpc, app)?;

	// application logic

	// create a new student certificate and upload it
	let key = Certificate::gen_cert_key();
	let (id, certificate) = Certificate::from_config(&config);
	let certificate = certificate.encrypt(&key);

	// upload certificate
	println!("Uploading certificate...");
	let tx_hash = capsules.put(id, certificate).await?;

	// print results
	println!("Certificate uploaded with tx hash: 0x{}", hex::encode(tx_hash.as_bytes()));
	println!("Certificate encryption key: 0x{}", hex::encode(key));

	Ok(())
}
