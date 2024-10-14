use super::{CapsulesApi, CapsulesConfig};
use crate::titanh::{
    capsules::calls::types::upload_capsule::App,
    runtime_types::primitives::common_types::BoundedString,
};
use anyhow::Result;
use codec::Encode;
use sp_core::{Blake2Hasher, Hasher, H256};

pub const CAPSULE_ID_PREFIX: &[u8] = b"cpsl";

impl CapsulesApi<'_> {
    // ensures the api configuration is set
    pub fn ensure_config(&self) -> Result<&CapsulesConfig> {
        self.config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Capsules API configuration is not provided"))
    }

    pub fn compute_capsule_id<Id: Encode>(&self, id: Id, app: App) -> H256 {
        let mut ids = Vec::new();

        ids.extend_from_slice(CAPSULE_ID_PREFIX);
        ids.extend_from_slice(&app.encode());
        ids.extend_from_slice(&id.encode());

        Blake2Hasher::hash(&ids[..])
    }
}

pub fn convert_bounded_str(bounded_str: BoundedString) -> Result<String> {
    let bounded = bounded_str.0 .0.to_vec();
    let str = std::str::from_utf8(&bounded)?;

    Ok(str.to_string())
}
