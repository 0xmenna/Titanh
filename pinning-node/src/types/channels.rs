use crate::types::events::NodeEvent;
use anyhow::Result;
use api::common_types::BlockNumber;
use tokio::sync::mpsc::{
	channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};

pub fn build_pool_handles() -> (PoolWritingHandle, PoolReadingHandle) {
	// Channel to send a single block number
	let (tx_block, rx_block) = channel(1);
	// Channel to handle events
	let (tx_events, rx_events) = unbounded_channel();

	(
		PoolWritingHandle { is_block_number_sent: false, tx_block, tx_events },
		PoolReadingHandle { rx_block, rx_events },
	)
}

#[derive(Clone)]
pub struct PoolWritingHandle {
	is_block_number_sent: bool,
	tx_block: Sender<BlockNumber>,
	tx_events: UnboundedSender<NodeEvent>,
}

impl PoolWritingHandle {
	pub async fn send_block_number(&mut self, number: BlockNumber) -> Result<()> {
		self.tx_block.send(number).await?;
		self.is_block_number_sent = true;

		Ok(())
	}

	pub fn send_event(&mut self, event: NodeEvent) -> Result<()> {
		self.tx_events.send(event).map_err(|e| e.into())
	}

	pub fn is_block_number_sent(&self) -> bool {
		self.is_block_number_sent
	}
}

pub struct PoolReadingHandle {
	rx_block: Receiver<BlockNumber>,
	rx_events: UnboundedReceiver<NodeEvent>,
}

impl PoolReadingHandle {
	pub fn receive_block_number(&mut self) -> Result<BlockNumber> {
		let block_number = self.rx_block.blocking_recv();

		if let Some(block_number) = block_number {
			self.rx_block.close();

			Ok(block_number)
		} else {
			Err(anyhow::anyhow!("Channel is already closed"))
		}
	}

	pub async fn receive_events(&mut self) -> Option<NodeEvent> {
		self.rx_events.recv().await
	}
}
