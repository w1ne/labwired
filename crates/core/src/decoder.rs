use crate::SimResult;
use crate::SimulationError;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    MovImm { rd: u8, imm: u8 }, // MOV Rd, #imm8
    Branch { offset: i32 },     // B <label>
    Unknown(u16),
}

/// Decodes a 16-bit Thumb instruction
pub fn decode_thumb_16(opcode: u16) -> Instruction {
    // Thumb-16 encoding reference (simplified)
    
    // NOP: 1011 1111 0000 0000 -> 0xBF00
    if opcode == 0xBF00 {
        return Instruction::Nop;
    }
    
    // MOV Rd, #imm8
    // Format: 0010 0ddd iiii iiii (T1 encoding)
    // Mask:   1110 0000 0000 0000 (0xE000) -> compare 0x2000
    if (opcode & 0xE000) == 0x2000 {
        let rd = ((opcode >> 8) & 0x7) as u8;
        let imm = (opcode & 0xFF) as u8;
        return Instruction::MovImm { rd, imm };
    }
    
    // B <label> (Conditional branch) - 1101 ...
    // B <label> (Unconditional) - 1110 0... (T2 encoding)
    // Let's implement Unconditional Branch T2: 1110 0iii iiii iiii
    if (opcode & 0xF800) == 0xE000 {
        let mut offset = (opcode & 0x7FF) as i32;
        // Sign extend 11-bit offset
        if (offset & 0x400) != 0 {
            offset |= !0x7FF;
        }
        // Target = PC + 4 + (offset << 1)
        // We will just return the raw offset shifted
        return Instruction::Branch { offset: offset << 1 };
    }

    Instruction::Unknown(opcode)
}
