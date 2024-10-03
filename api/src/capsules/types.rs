use crate::{
    common_types::ConsistencyLevel, titanh::runtime_types::pallet_capsules::types::FollowersStatus,
};
use sp_core::H256;

pub type CapsuleKey = H256;

const DEFAULT_CAPSULE_RETENTION_BLOCKS: u32 = 864_000; // 1 month

pub struct PutCapsuleOpts {
    pub retention_blocks: Option<u32>,
    pub followers_status: Option<FollowersStatus>,
    pub level: ConsistencyLevel,
}

impl Default for PutCapsuleOpts {
    fn default() -> Self {
        Self {
            retention_blocks: Some(DEFAULT_CAPSULE_RETENTION_BLOCKS),
            followers_status: Some(FollowersStatus::None),
            level: Default::default(),
        }
    }
}

impl PutCapsuleOpts {
    pub fn unwrap_fields_or_default(self) -> (u32, FollowersStatus, ConsistencyLevel) {
        (
            self.retention_blocks
                .unwrap_or(DEFAULT_CAPSULE_RETENTION_BLOCKS),
            self.followers_status.unwrap_or(FollowersStatus::None),
            self.level,
        )
    }
}

#[derive(Default)]
pub struct GetCapsuleOpts {
    pub from_finalized_state: bool,
}

#[derive(Default)]
pub struct UpdateCapsuleOpts {
    pub level: ConsistencyLevel,
}
