use crate::{Cpu, Bus, SimResult};

#[derive(Debug, Default)]
pub struct CortexM {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub sp: u32, // R13
    pub lr: u32, // R14
    pub pc: u32, // R15
    pub xpsr: u32,
}

impl CortexM {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Cpu for CortexM {
    fn reset(&mut self) {
        // In real Cortex-M, we'd read initial SP from 0x0000_0000 and PC from 0x0000_0004
        self.pc = 0x0000_0000;
        self.sp = 0x2000_0000; // Mock stack pointer
    }

    fn step(&mut self, bus: &mut dyn Bus) -> SimResult<()> {
        // Fetch
        let instruction_byte = bus.read_u8(self.pc as u64)?;
        
        // Mock Decode & Execute
        // For iteration 1, we just increment PC to simulate progress
        // and maybe do a NOP
        
        tracing::debug!("Executing at PC={:#x}, Opcode={:#x}", self.pc, instruction_byte);
        
        self.pc += 1; // Very wrong for ARM (instructions are 2 or 4 bytes), but okay for stub.
        
        Ok(())
    }
}
