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
    pub fn async_write(&mut self, fd: RawFd, buf: &[u8], user_data: u64) -> Result<()> {
        let e = opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as u32).build().user_data(user_data);
        unsafe { self.write_ring.submission().push(&e).map_err(|_| TeeError::IoUring("write ring full".into()))?; }
        Ok(())
    }
    pub fn flush_writes(&mut self, min: usize) -> Result<()> {
        self.write_ring.submit_and_wait(min).map_err(|e| TeeError::IoUring(e.to_string()))?;
        Ok(())
    }
    pub fn write_ring_fd(&self) -> RawFd { self.write_ring.as_raw_fd() }
    pub fn read_ring_fd(&self) -> RawFd { self.read_ring.as_raw_fd() }
}