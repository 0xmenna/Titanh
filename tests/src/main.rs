use anyhow::Result;
use mocks::{MockApi, CHAIN_ENDPOINT, KEYS_DIR, KEYS_LEAVE_DIR, NUM_CAPSULES, SEED_PRHASE};
use std::fs;
use titan_api::{CapsulesBatch, TitanhApiBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    // Create output directories
    fs::create_dir_all(KEYS_DIR)?;
    fs::create_dir_all(KEYS_LEAVE_DIR)?;

    let api = TitanhApiBuilder::rpc(CHAIN_ENDPOINT)
        .seed(SEED_PRHASE)
        .build()
        .await?;

    let mut mock_api = MockApi::mock_from_api(&api);

    let data = String::from("This is just some tesing data to upload to IPFS: ");

    let mut batch = CapsulesBatch::new();
    for i in 0..NUM_CAPSULES {
        let mut value = data.clone();
        value.push_str(&i.to_string());

        let id = i as u32;
        mock_api.assign_key_to_replicas(id);

        batch.insert((id, value));
    }

    mock_api.capsules.put_batch_async(batch).await?;

    mock_api.display_assigned_keys(KEYS_DIR);
    mock_api.display_leave_simulation(KEYS_LEAVE_DIR);

    Ok(())
}

mod mocks;
