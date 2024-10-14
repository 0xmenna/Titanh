use super::traits::Dispatcher;
use crate::{db::checkpointing::DbCheckpoint, types::events::CheckpointEvent};
use anyhow::Result;

impl Dispatcher<CheckpointEvent<'_>, ()> for DbCheckpoint {
    fn dispatch(&self, event: CheckpointEvent) -> Result<()> {
        let block_num = event.block_num;
        let rows = event.table_rows;
        let pin_counts = event.pin_counts;

        self.commit_checkpoint(block_num, rows, pin_counts)?;

        Ok(())
    }
}
