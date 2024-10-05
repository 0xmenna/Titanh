use crate::types::events::NodeEvent;
use anyhow::Result;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub fn build_pool_handles() -> (PoolWritingHandle, PoolReadingHandle) {
    // Channel to handle events
    let (tx_events, rx_events) = unbounded_channel();

    (
        PoolWritingHandle { tx_events },
        PoolReadingHandle { rx_events },
    )
}

#[derive(Clone)]
pub struct PoolWritingHandle {
    tx_events: UnboundedSender<NodeEvent>,
}

impl PoolWritingHandle {
    pub fn send_event(&mut self, event: NodeEvent) -> Result<()> {
        self.tx_events.send(event).map_err(|e| e.into())
    }
}

pub struct PoolReadingHandle {
    rx_events: UnboundedReceiver<NodeEvent>,
}

impl PoolReadingHandle {
    pub async fn receive_events(&mut self) -> Option<NodeEvent> {
        self.rx_events.recv().await
    }
}
