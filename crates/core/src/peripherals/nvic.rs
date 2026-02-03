use crate::{SimResult, Peripheral};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Shared state for NVIC registers.
#[derive(Debug)]
pub struct NvicState {
    pub iser: [AtomicU32; 8],
    pub ispr: [AtomicU32; 8],
}

impl Default for NvicState {
    fn default() -> Self {
        Self {
            iser: [
                AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
                AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
            ],
            ispr: [
                AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
                AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
            ],
        }
    }
}

/// Nested Vectored Interrupt Controller (NVIC) mock.
#[derive(Debug, Clone)]
pub struct Nvic {
    pub state: Arc<NvicState>,
}

impl Nvic {
    pub fn new(state: Arc<NvicState>) -> Self {
        Self { state }
    }

    pub fn is_enabled(&self, irq: u32) -> bool {
        if irq < 16 { return true; }
        let idx = ((irq - 16) / 32) as usize;
        let bit = (irq - 16) % 32;
        if idx < 8 {
            (self.state.iser[idx].load(Ordering::SeqCst) & (1 << bit)) != 0
        } else {
            false
        }
    }
}

impl Peripheral for Nvic {
    fn read(&self, offset: u64) -> SimResult<u8> {
        let reg_idx = (offset / 4) as usize;
        let byte_offset = (offset % 4) as usize;
        
        let val = if offset < 0x20 {
            // ISER0-7
            self.state.iser[reg_idx].load(Ordering::SeqCst)
        } else if offset >= 0x100 && offset < 0x120 {
            // ISPR0-7
            let real_idx = (offset - 0x100) / 4;
            self.state.ispr[real_idx as usize].load(Ordering::SeqCst)
        } else {
            0
        };
        
        Ok(((val >> (byte_offset * 8)) & 0xFF) as u8)
    }

    fn write(&mut self, offset: u64, value: u8) -> SimResult<()> {
        let reg_idx = (offset / 4) as usize;
        let byte_offset = (offset % 4) as usize;
        let mask = (value as u32) << (byte_offset * 8);

        if offset < 0x20 {
            // ISER: Writing 1 sets the enable bit
            self.state.iser[reg_idx].fetch_or(mask, Ordering::SeqCst);
            tracing::info!("NVIC: ISER[{}] set to {:#x}", reg_idx, self.state.iser[reg_idx].load(Ordering::SeqCst));
        } else if offset >= 0x80 && offset < 0xA0 {
            // ICER: Writing 1 clears the enable bit
            let real_idx = reg_idx - 0x80/4;
            self.state.iser[real_idx].fetch_and(!mask, Ordering::SeqCst);
            tracing::info!("NVIC: ISER[{}] cleared to {:#x}", real_idx, self.state.iser[real_idx].load(Ordering::SeqCst));
        } else if offset >= 0x100 && offset < 0x120 {
            // ISPR: Writing 1 sets the pending bit
            let real_idx = reg_idx - 0x100/4;
            self.state.ispr[real_idx].fetch_or(mask, Ordering::SeqCst);
            tracing::info!("NVIC: ISPR[{}] set to {:#x}", real_idx, self.state.ispr[real_idx].load(Ordering::SeqCst));
        } else if offset >= 0x180 && offset < 0x1A0 {
            // ICPR: Writing 1 clears the pending bit
            let real_idx = reg_idx - 0x180/4;
            self.state.ispr[real_idx].fetch_and(!mask, Ordering::SeqCst);
        }
        
        Ok(())
    }
}
