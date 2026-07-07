use crate::error::{Result, TeeError};
use io_uring::{opcode, types, IoUring};
use std::os::unix::io::{AsRawFd, RawFd};

pub struct MultiQueueEngine {
    pub write_ring: IoUring,
    pub read_ring: IoUring,
}

impl MultiQueueEngine {
    pub fn new(write_entries: u32, read_entries: u32) -> Result<Self> {
        Ok(MultiQueueEngine {
            write_ring: IoUring::new(write_entries).map_err(|e| TeeError::IoUring(e.to_string()))?,
            read_ring: IoUring::new(read_entries).map_err(|e| TeeError::IoUring(e.to_string()))?,
        })
    }
    pub fn write_ring_fd(&self) -> RawFd { self.write_ring.as_raw_fd() }
    pub fn read_ring_fd(&self) -> RawFd { self.read_ring.as_raw_fd() }
}