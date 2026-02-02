use crate::{Cpu, Bus, SimResult};
use crate::decoder::{self, Instruction};

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
        self.pc = 0x0000_0000;
        self.sp = 0x2000_0000;
    }

    fn step(&mut self, bus: &mut dyn Bus) -> SimResult<()> {
        // Fetch 16-bit thumb instruction
        // Note: PC in Thumb mode usually points to current instruction + 4 due to pipeline.
        // For simulation, we treat PC as fetch address for now.
        
        let fetch_pc = self.pc & !1;
        let opcode = bus.read_u16(fetch_pc as u64)?;
        
        // Decode
        let instruction = decoder::decode_thumb_16(opcode);
        
        tracing::debug!("PC={:#x}, Opcode={:#04x}, Instr={:?}", self.pc, opcode, instruction);
        
        // Execute
        let mut pc_increment = 2; // Default for 16-bit instruction
        
        match instruction {
            Instruction::Nop => {
                // Do nothing
            },
            Instruction::MovImm { rd, imm } => {
                // R[d] = imm
                match rd {
                    0 => self.r0 = imm as u32,
                    1 => self.r1 = imm as u32,
                    2 => self.r2 = imm as u32,
                    3 => self.r3 = imm as u32,
                    _ => tracing::warn!("Unimplemented register R{}", rd),
                }
            },
            Instruction::Branch { offset } => {
                 // PC = PC + 4 + offset
                 // Target = (CurrentPC + 4) + offset
                 let target = (self.pc as i32 + 4 + offset) as u32;
                 self.pc = target;
                 pc_increment = 0; // Don't increment after branch
            },
            Instruction::Unknown(_) => {
                tracing::warn!("Unknown instruction at {:#x}", self.pc);
            }
        }
        
        self.pc = self.pc.wrapping_add(pc_increment);
        
        Ok(())
    }
}
