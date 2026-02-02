use crate::{SimResult, SimulationError};
use crate::memory::LinearMemory;

use crate::peripherals::uart::Uart;

pub struct SystemBus {
    pub flash: LinearMemory,
    pub ram: LinearMemory,
    pub uart: crate::peripherals::uart::Uart,
    pub systick: crate::peripherals::systick::Systick,
}

impl SystemBus {
    pub fn new(flash_size: usize, ram_size: usize) -> Self {
        Self {
            flash: LinearMemory::new(flash_size, 0x0000_0000),
            ram: LinearMemory::new(ram_size, 0x2000_0000),
            uart: crate::peripherals::uart::Uart::new(),
            systick: crate::peripherals::systick::Systick::new(),
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
        
        // SysTick: 0xE000_E010
        if addr >= 0xE000_E010 && addr < 0xE000_E020 {
             // SysTick registers are 32-bit, handle byte reads by reading u32 and shifting
             let reg_offset = (addr - 0xE000_E010) & !3;
             let val = self.systick.read(reg_offset)?;
             let byte_shift = (addr % 4) * 8;
             return Ok(((val >> byte_shift) & 0xFF) as u8);
        }
        
        Err(SimulationError::MemoryViolation(addr))
    }

    fn write_u8(&mut self, addr: u64, value: u8) -> SimResult<()> {
        if self.ram.write_u8(addr, value) {
            return Ok(());
        }
        
        if self.flash.write_u8(addr, value) {
            return Ok(());
        }
        
        // UART Stub
        if addr >= 0x4000_C000 && addr < 0x4000_D000 {
            return self.uart.write(addr - 0x4000_C000, value);
        }
        
        // SysTick: 0xE000_E010
        if addr >= 0xE000_E010 && addr < 0xE000_E020 {
             // Simple byte-to-word write for now (mostly SysTick expects word access)
             let reg_offset = (addr - 0xE000_E010) & !3;
             // Read current word, modify byte, write back
             let mut val = self.systick.read(reg_offset)?;
             let byte_shift = (addr % 4) * 8;
             val &= !(0xFF << byte_shift);
             val |= (value as u32) << byte_shift;
             return self.systick.write(reg_offset, val);
        }

        Err(SimulationError::MemoryViolation(addr))
    }
}
