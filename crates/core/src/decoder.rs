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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_nop() {
        assert_eq!(decode_thumb_16(0xBF00), Instruction::Nop);
    }

    #[test]
    fn test_decode_mov_imm() {
        // MOV R0, #42 -> 0x202A
        assert_eq!(
            decode_thumb_16(0x202A),
            Instruction::MovImm { rd: 0, imm: 42 }
        );
        // MOV R3, #255 -> 0x23FF
        assert_eq!(
            decode_thumb_16(0x23FF),
            Instruction::MovImm { rd: 3, imm: 255 }
        );
    }

    #[test]
    fn test_decode_branch() {
        // B <label> (Unconditional)
        // Opcode 0xE000 | 11-bit offset
        // Offset is sign extended and shifted left by 1.
        
        // Positive offset: +2 (in instructions) -> +4 bytes
        // Encoding: i=2 -> 0xE002
        // Target = PC + 4 + (2 << 1) = PC + 8
        assert_eq!(
            decode_thumb_16(0xE002),
            Instruction::Branch { offset: 4 }
        );
        
        // Negative offset: -2 (in instructions) -> -4 bytes
        // i = -2 = 0x7FE (11-bit two's complement)
        // Opcode = 0xE000 | 0x7FE = 0xE7FE
        // Target = PC + 4 + (-2 << 1) = PC
        assert_eq!(
            decode_thumb_16(0xE7FE),
            Instruction::Branch { offset: -4 }
        );
    }

    #[test]
    fn test_decode_unknown() {
        assert_eq!(decode_thumb_16(0xFFFF), Instruction::Unknown(0xFFFF));
    }
}
