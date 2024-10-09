use anyhow::Result;
use ipfs_api_backend_hyper::{request::Add, IpfsApi, IpfsClient, TryFromUri};
use pinning::{
    checkpointing::{Checkpoint, DbCheckpoint},
    FaultTolerantKeyTable,
};
use sp_core::H256;
use std::io::Cursor;

pub fn read_node_checkpoint_from_db(
    rep_factor: u32,
    node_id: H256,
) -> Result<Checkpoint> {
    let db = DbCheckpoint::from_values(rep_factor, node_id, false);
    let checkpoint = db.get_checkpoint()?;
    Ok(checkpoint)
}

/// Upload the keytable rows to IPFS and return the IPFS cids of the uploaded rows
pub async fn upload_keytable_to_ipfs(
    ipfs_rpc: &str,
    keytable: FaultTolerantKeyTable,
) -> Result<Vec<Vec<u8>>> {
    let ipfs = IpfsClient::from_str(ipfs_rpc)?;

    let encoded_rows = keytable.encoded_rows();

    let mut rows_cids = Vec::new();
    for encoded_row in encoded_rows {
        let data = Cursor::new(encoded_row);

        // Do not pin the data
        let mut add_opts = Add::default();
        add_opts.pin = Some(false);
        // Add the data to IPFS
        let ipfs_res = ipfs.add_with_options(data, add_opts).await?;

        let cid = ipfs_res.hash.as_bytes().to_vec();
        rows_cids.push(cid);
    }

    Ok(rows_cids)
}
