use codec::{Decode, Encode};
use primitives::BlockNumber;

#[derive(Encode, Decode)]
pub struct PinningCheckpoint {
	block_number: BlockNumber,
	processed_events: u32,
}
