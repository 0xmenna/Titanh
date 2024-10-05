use crate::common_types::ConsistencyLevel;
use codec::Encode;
use sp_core::H256;

pub type CapsuleKey = H256;

const DEFAULT_CAPSULE_RETENTION_BLOCKS: u32 = 864_000; // 1 month

pub struct PutCapsuleOpts {
    pub retention_blocks: Option<u32>,
    pub level: ConsistencyLevel,
}

impl Default for PutCapsuleOpts {
    fn default() -> Self {
        Self {
            retention_blocks: Some(DEFAULT_CAPSULE_RETENTION_BLOCKS),
            level: Default::default(),
        }
    }
}

impl PutCapsuleOpts {
    pub fn unwrap_fields_or_default(&self) -> (u32, ConsistencyLevel) {
        (
            self.retention_blocks
                .unwrap_or(DEFAULT_CAPSULE_RETENTION_BLOCKS),
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

pub struct CapsulesBatch<Id, Value> {
    capsules: Vec<(Id, Value)>,
}

impl<Id: Encode, Value: Encode> CapsulesBatch<Id, Value> {
    pub fn new() -> Self {
        Self {
            capsules: Vec::new(),
        }
    }

    pub fn insert(&mut self, pair: (Id, Value)) {
        self.capsules.push(pair);
    }
}

impl<Id, Value> IntoIterator for CapsulesBatch<Id, Value> {
    type Item = (Id, Value);
    type IntoIter = std::vec::IntoIter<(Id, Value)>;

    fn into_iter(self) -> Self::IntoIter {
        self.capsules.into_iter()
    }
}
