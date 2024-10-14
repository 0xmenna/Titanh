use std::str::FromStr;

use anyhow::Result;
use sp_core::H256;

pub fn load_env() -> Result<(String, String, Option<(H256, H256)>)> {
    let collector_seed = std::env::var("COLLECTOR_SEED")?;
    let rpc_url = std::env::var("RPC_URL")?;
    let key_range = std::env::var("KEY_RANGE")?;

    let key_range = if key_range.is_empty() {
        None
    } else {
        let keys: Vec<&str> = key_range.split(',').collect();
        let start = H256::from_str(keys[0])?;
        let end = H256::from_str(keys[1])?;
        Some((start, end))
    };

    Ok((collector_seed, rpc_url, key_range))
}
