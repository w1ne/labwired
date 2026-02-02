pub mod cpu;
pub mod memory;
pub mod bus;
pub mod decoder;

mod tests;

#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("Memory access violation at {0:#x}")]
    MemoryViolation(u64),
    #[error("Instruction decoding error at {0:#x}")]
    DecodeError(u64),
}

pub type SimResult<T> = Result<T, SimulationError>;

/// Trait representing a CPU architecture
pub trait Cpu {
    fn reset(&mut self);
    fn step(&mut self, bus: &mut dyn Bus) -> SimResult<()>;
}

/// Trait representing the system bus
pub trait Bus {
    fn read_u8(&self, addr: u64) -> SimResult<u8>;
    fn write_u8(&mut self, addr: u64, value: u8) -> SimResult<()>;
    
    fn read_u16(&self, addr: u64) -> SimResult<u16> {
        let b0 = self.read_u8(addr)? as u16;
        let b1 = self.read_u8(addr + 1)? as u16;
        // Little Endian
        Ok(b0 | (b1 << 8))
    }

    fn read_u32(&self, addr: u64) -> SimResult<u32> {
        let b0 = self.read_u8(addr)? as u32;
        let b1 = self.read_u8(addr + 1)? as u32;
        let b2 = self.read_u8(addr + 2)? as u32;
        let b3 = self.read_u8(addr + 3)? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }
}

pub struct Machine {
    pub cpu: cpu::CortexM,
    pub bus: bus::SystemBus,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CortexM::new(),
            bus: bus::SystemBus::new(1024 * 1024, 128 * 1024), // 1MB Flash, 128KB RAM mock
        }
    }
    
    pub fn load_firmware(&mut self, image: &memory::ProgramImage) -> SimResult<()> {
        for segment in &image.segments {
            // Try loading into Flash first
            if !self.bus.flash.load_from_segment(segment) {
                // If not flash, try RAM? Or just warn?
                // For now, let's assume everything goes to Flash or RAM mapped spaces
                 if !self.bus.ram.load_from_segment(segment) {
                     tracing::warn!("Failed to load segment at {:#x} - outside of memory map", segment.start_addr);
                 }
            }
        }
        
        // simple vector table reset (Mock)
        // Real Cortex-M: Read SP from 0x0, PC from 0x4
        if let Ok(sp) = self.bus.read_u32(0x0000_0000) {
            self.cpu.sp = sp;
        }
        if let Ok(pc) = self.bus.read_u32(0x0000_0004) {
             self.cpu.pc = pc;
        } else {
            // Fallback to entry point from ELF if raw binary load failed mostly
            self.cpu.pc = image.entry_point as u32;
        }
        
        Ok(())
    }
    
    pub fn step(&mut self) -> SimResult<()> {
        self.cpu.step(&mut self.bus)
    }
}
