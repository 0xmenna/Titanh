use super::{chain::titanh::app_registrar::events, pinning::PinningCapsuleEvent};
use anyhow::Result;
use primitives::BlockNumber;
use tokio::sync::mpsc::{
	channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};

pub fn build_channels() -> (PinningWritingHandles, PinningReadingHandles) {
	// Channel to send a single block number
	let (tx_block, rx_block) = channel(1);
	// Channel to handle events
	let (tx_events, rx_events) = unbounded_channel();

	(
		PinningWritingHandles { is_block_number_sent: false, tx_block, tx_events },
		PinningReadingHandles { rx_block, rx_events },
	)
}

#[derive(Clone)]
pub struct PinningWritingHandles {
	is_block_number_sent: bool,
	tx_block: Sender<BlockNumber>,
	tx_events: UnboundedSender<PinningCapsuleEvent>,
}

impl PinningWritingHandles {
	pub async fn send_block_number(&mut self, number: BlockNumber) -> Result<()> {
		self.tx_block.send(number).await?;
		self.is_block_number_sent = true;

		Ok(())
	}

	pub fn send_event(&mut self, event: PinningCapsuleEvent) -> Result<()> {
		self.tx_events.send(event).map_err(|e| e.into())
	}

	pub fn is_block_number_sent(&self) -> bool {
		self.is_block_number_sent
	}
}

pub struct PinningReadingHandles {
	rx_block: Receiver<BlockNumber>,
	rx_events: UnboundedReceiver<PinningCapsuleEvent>,
}

impl PinningReadingHandles {
	pub fn receive_block_number(&mut self) -> Result<BlockNumber> {
		let block_number = self.rx_block.blocking_recv();

		if let Some(block_number) = block_number {
			self.rx_block.close();

			Ok(block_number)
		} else {
			Err(anyhow::anyhow!("Channel is already closed"))
		}
	}

	pub async fn rx_event(&self) -> &UnboundedReceiver<PinningCapsuleEvent> {
		&self.rx_events
	}
}
