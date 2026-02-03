use crate::{SimResult, Peripheral};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// System Control Block (SCB) mock.
/// Handles VTOR relocation and other system-level controls.
#[derive(Debug)]
pub struct Scb {
    vtor: Arc<AtomicU32>,
}

impl Scb {
    pub fn new(vtor: Arc<AtomicU32>) -> Self {
        Self { vtor }
    }
}

impl Peripheral for Scb {
    fn read(&self, offset: u64) -> SimResult<u8> {
        match offset {
            0x08 => Ok((self.vtor.load(Ordering::SeqCst) & 0xFF) as u8),
            0x09 => Ok(((self.vtor.load(Ordering::SeqCst) >> 8) & 0xFF) as u8),
            0x0A => Ok(((self.vtor.load(Ordering::SeqCst) >> 16) & 0xFF) as u8),
            0x0B => Ok(((self.vtor.load(Ordering::SeqCst) >> 24) & 0xFF) as u8),
            _ => Ok(0),
        }
    }

    fn write(&mut self, offset: u64, value: u8) -> SimResult<()> {
        let mut curr = self.vtor.load(Ordering::SeqCst);
        match offset {
            0x08 => curr = (curr & 0xFFFFFF00) | (value as u32),
            0x09 => curr = (curr & 0xFFFF00FF) | ((value as u32) << 8),
            0x0A => curr = (curr & 0xFF00FFFF) | ((value as u32) << 16),
            0x0B => curr = (curr & 0x00FFFFFF) | ((value as u32) << 24),
            _ => return Ok(()),
        }
        self.vtor.store(curr, Ordering::SeqCst);
        tracing::debug!("SCB: VTOR updated to {:#x}", curr);
        Ok(())
    }
}
