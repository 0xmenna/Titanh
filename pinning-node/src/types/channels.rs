use super::{chain::titanh::app_registrar::events, pinning::PinningCapsuleEvent};
use anyhow::Result;
use primitives::BlockNumber;
use tokio::sync::mpsc::{
	channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};

pub struct PinningChannels {
	is_block_number_sent: bool,

	rx_block: Receiver<BlockNumber>,
	tx_block: Sender<BlockNumber>,

	rx_event: UnboundedReceiver<PinningCapsuleEvent>,
	tx_event: UnboundedSender<PinningCapsuleEvent>,
}

impl PinningChannels {
	pub fn new() -> Self {
		let (tx_block, rx_block) = channel(1);
		let (tx_event, rx_event) = unbounded_channel();
		Self { is_block_number_sent: false, rx_block, tx_block, rx_event, tx_event }
	}

	pub async fn send_block_number(&mut self, number: BlockNumber) -> Result<()> {
		self.tx_block.send(number).await?;
		self.is_block_number_sent = true;

		Ok(())
	}

	pub async fn receive_block_number(&mut self) -> Option<BlockNumber> {
		self.rx_block.recv().await
	}

	pub async fn rx_event(&self) -> &UnboundedReceiver<PinningCapsuleEvent> {
		&self.rx_event
	}

	pub fn send_event(&mut self, event: PinningCapsuleEvent) -> Result<()> {
		self.tx_event.send(event).map_err(|e| e.into())
	}

	pub fn is_block_number_sent(&self) -> bool {
		self.is_block_number_sent
	}
}
