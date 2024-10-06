use super::traits::AsyncMutableDispatcher;
use crate::ipfs::client::IpfsClient;
use crate::types::events::{PinEventFromLeaveNode, PinningEvent, UnpinningEvent};
use crate::types::keytable::TableRow;
use anyhow::Result;
use async_trait::async_trait;
use codec::Decode;

#[async_trait(?Send)]
impl AsyncMutableDispatcher<PinningEvent, ()> for IpfsClient {
    async fn async_dispatch(&mut self, pinning_event: PinningEvent) -> Result<()> {
        match pinning_event {
            PinningEvent::Pin { cid } => {
                self.pin_add(&cid).await;
            }

            PinningEvent::UpdatePin { old_cid, new_cid } => {
                self.pin_remove(&old_cid).await?;
                self.pin_add(&new_cid).await;
            }

            PinningEvent::RemovePin { cid } => {
                self.pin_remove(&cid).await?;
            }
        };

        Ok(())
    }
}

#[async_trait(?Send)]
impl AsyncMutableDispatcher<PinEventFromLeaveNode, TableRow> for IpfsClient {
    async fn async_dispatch(&mut self, event: PinEventFromLeaveNode) -> Result<TableRow> {
        let (cid, pin_batch) = event;

        let transferred_row = self.get(cid).await?;
        let row = TableRow::decode(&mut &transferred_row[..])?;

        for cid in row.values() {
            self.pin_add(cid).await;
        }

        for pin in pin_batch {
            self.async_dispatch(pin).await.unwrap();
        }

        Ok(row)
    }
}

#[async_trait(?Send)]
impl AsyncMutableDispatcher<UnpinningEvent, ()> for IpfsClient {
    async fn async_dispatch(&mut self, unpinning_event: UnpinningEvent) -> Result<()> {
        for cid in unpinning_event.values() {
            self.pin_remove(cid).await?;
        }

        Ok(())
    }
}
