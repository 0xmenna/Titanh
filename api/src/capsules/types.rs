use crate::titanh::runtime_types::pallet_capsules::types::FollowersStatus;

const DEFAULT_CAPSULE_RETENTION_BLOCKS: u32 = 864_000; // 1 month

pub struct PutCapsuleOpts {
	pub retention_blocks: Option<u32>,
	pub followers_status: Option<FollowersStatus>,
	pub wait_finalization: bool,
}

impl Default for PutCapsuleOpts {
	fn default() -> Self {
		Self {
			retention_blocks: Some(DEFAULT_CAPSULE_RETENTION_BLOCKS),
			followers_status: Some(FollowersStatus::None),
			wait_finalization: false,
		}
	}
}

impl PutCapsuleOpts {
	pub fn unwrap_fields_or_default(self) -> (u32, FollowersStatus, bool) {
		(
			self.retention_blocks.unwrap_or(DEFAULT_CAPSULE_RETENTION_BLOCKS),
			self.followers_status.unwrap_or(FollowersStatus::None),
			self.wait_finalization,
		)
	}
}
