use codec::Encode;
use sp_core::{Blake2Hasher, Hasher, H256};

const CAPSULE_ID_PREFIX: &[u8] = b"cpsl";

pub fn compute_capsule_id(metadata: Vec<u8>, app_id: u32) -> H256 {
    let mut ids = Vec::new();

    ids.extend_from_slice(CAPSULE_ID_PREFIX);
    ids.extend_from_slice(&app_id.encode());
    ids.extend_from_slice(&metadata);

    Blake2Hasher::hash(&ids[..])
}
