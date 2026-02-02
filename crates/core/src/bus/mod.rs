use crate::{SimResult, SimulationError};
use crate::memory::LinearMemory;

use crate::peripherals::uart::Uart;

pub struct SystemBus {
    pub flash: LinearMemory,
    pub ram: LinearMemory,
    pub uart: Uart,
}

impl SystemBus {
    pub fn new(flash_size: usize, ram_size: usize) -> Self {
        Self {
            flash: LinearMemory::new(flash_size, 0x0000_0000),
            ram: LinearMemory::new(ram_size, 0x2000_0000),
            uart: Uart::new(),
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
        // UART Stub: 0x4000_C000
        // Simple check for now
        if addr >= 0x4000_C000 && addr < 0x4000_D000 {
            return self.uart.read(addr - 0x4000_C000);
        }
        
        Err(SimulationError::MemoryViolation(addr))
    }

    fn write_u8(&mut self, addr: u64, value: u8) -> SimResult<()> {
        if self.ram.write_u8(addr, value) {
            return Ok(());
        }
        
        // UART Stub
        if addr >= 0x4000_C000 && addr < 0x4000_D000 {
            return self.uart.write(addr - 0x4000_C000, value);
        }

        if self.flash.read_u8(addr).is_some() {
             return Err(SimulationError::MemoryViolation(addr)); 
        }
        
        Err(SimulationError::MemoryViolation(addr))
    }
}
