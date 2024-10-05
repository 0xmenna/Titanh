use crate::{
    types::channels::{self, PoolReadingHandle, PoolWritingHandle},
    utils::ref_builder::{self, MutableRef},
};

pub struct NodeEventsPool {
    // Handles to reading channels (to recive the block number and events)
    reading_handle: PoolReadingHandle,
    // Handles to writing channels (to send the block number and events)
    writing_handle: PoolWritingHandle,
}

impl NodeEventsPool {
    pub fn new() -> Self {
        // Create handles to write and read from the channel
        let (writing_handle, reading_handle) = channels::build_pool_handles();

        Self {
            reading_handle,
            writing_handle,
        }
    }

    pub fn read_handle(&mut self) -> &mut PoolReadingHandle {
        &mut self.reading_handle
    }

    pub fn write_handle(&self) -> PoolWritingHandle {
        self.writing_handle.clone()
    }

    pub fn mutable_ref(self) -> MutableRef<Self> {
        ref_builder::create_mutable_ref(self)
    }
}
