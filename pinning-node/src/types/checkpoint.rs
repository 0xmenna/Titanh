use primitives::BlockNumber;

pub struct PinningCheckpoint {
	block_number: BlockNumber,
	processed_events: u32,
}
