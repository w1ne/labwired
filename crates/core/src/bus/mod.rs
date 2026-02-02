use crate::{SimResult, SimulationError};
use crate::memory::LinearMemory;

pub struct SystemBus {
    pub flash: LinearMemory,
    pub ram: LinearMemory,
}

impl SystemBus {
    pub fn new(flash_size: usize, ram_size: usize) -> Self {
        Self {
            // Cortex-M usually has Flash at 0x0000_0000 and RAM at 0x2000_0000
            // We'll stick to that convention for now, but make it configurable later.
            flash: LinearMemory::new(flash_size, 0x0000_0000),
            ram: LinearMemory::new(ram_size, 0x2000_0000),
        }
    }
}

impl crate::Bus for SystemBus {
    fn read_u8(&self, addr: u64) -> SimResult<u8> {
        if let Some(byte) = self.flash.read_u8(addr) {
            return Ok(byte);
        }
        if let Some(byte) = self.ram.read_u8(addr) {
            return Ok(byte);
        }
        Err(SimulationError::MemoryViolation(addr))
    }

    fn write_u8(&mut self, addr: u64, value: u8) -> SimResult<()> {
        if self.ram.write_u8(addr, value) {
            return Ok(());
        }
        // Flash is usually read-only for direct writes (requires controller commands), 
        // but for now we'll imply it's ROM and error on write or just ignore?
        // Let's error for now to be safe.
        if self.flash.read_u8(addr).is_some() {
             // In real HW, this might be ignored or cause a HardFault
             return Err(SimulationError::MemoryViolation(addr)); 
        }
        
        Err(SimulationError::MemoryViolation(addr))
    }
}
