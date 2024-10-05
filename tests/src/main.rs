use std::collections::HashMap;

use anyhow::Result;
use mocks::{MockApi, CHAIN_ENDPOINT, NUM_CAPSULES, SEED_PRHASE};
use rand::Rng;
use titan_api::TitanhApiBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let api = TitanhApiBuilder::rpc(CHAIN_ENDPOINT)
        .seed(SEED_PRHASE)
        .build()
        .await;

    let mut mock_api = MockApi::mock_from_api(&api);

    let mut rng = rand::thread_rng();
    let mut used_keys = HashMap::new();

    let data = String::from("This is just some tesing data to upload to IPFS: ");
    for i in 0..NUM_CAPSULES {
        let mut value = data.clone();
        value.push_str(&i.to_string());

        let id = rng.gen::<u64>();
        if used_keys.contains_key(&id) {
            continue;
        }

        mock_api.put(id, value).await?;

        used_keys.insert(id, true);
    }

    mock_api.display_assigned_keys();

    Ok(())
}

mod mocks;
