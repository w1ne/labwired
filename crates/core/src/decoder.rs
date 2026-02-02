use crate::SimResult;
use crate::SimulationError;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    MovImm { rd: u8, imm: u8 },         // MOV Rd, #imm8
    Branch { offset: i32 },             // B <label>
    
    // Arithmetic & Logic
    AddReg { rd: u8, rn: u8, rm: u8 },  // ADD Rd, Rn, Rm
    AddImm3 { rd: u8, rn: u8, imm: u8 },// ADD Rd, Rn, #imm3
    AddImm8 { rd: u8, imm: u8 },        // ADD Rd, #imm8
    
    SubReg { rd: u8, rn: u8, rm: u8 },  // SUB Rd, Rn, Rm
    SubImm3 { rd: u8, rn: u8, imm: u8 },// SUB Rd, Rn, #imm3
    SubImm8 { rd: u8, imm: u8 },        // SUB Rd, #imm8
    
    CmpImm { rn: u8, imm: u8 },         // CMP Rn, #imm8
    
    And { rd: u8, rm: u8 },             // AND Rd, Rm
    Orr { rd: u8, rm: u8 },             // ORR Rd, Rm
    Eor { rd: u8, rm: u8 },             // EOR Rd, Rm
    Mvn { rd: u8, rm: u8 },             // MVN Rd, Rm
    
    // Memory
    LdrImm { rt: u8, rn: u8, imm: u8 }, // LDR Rt, [Rn, #imm] (imm is *4)
    StrImm { rt: u8, rn: u8, imm: u8 }, // STR Rt, [Rn, #imm] (imm is *4)
    LdrLit { rt: u8, imm: u8 },         // LDR Rt, [PC, #imm]

    // Stack
    Push { registers: u8, m: bool },    // PUSH {Rlist, LR?}
    Pop { registers: u8, p: bool },     // POP {Rlist, PC?}
    
    // Control Flow
    Bl { offset: i32 },                 // BL <label> (32-bit T1+T2)
    Bx { rm: u8 },                      // BX Rm
    
    Unknown(u16),
    // Intermediate state for 32-bit instruction (First half of BL)
    BlPrefix(u16),
}

/// Decodes a 16-bit Thumb instruction
pub fn decode_thumb_16(opcode: u16) -> Instruction {
    // 1. Move Immediate (T1): 0010 0ddd iiii iiii
    if (opcode & 0xE000) == 0x2000 {
        let op = (opcode >> 11) & 0x3;
        let rd = ((opcode >> 8) & 0x7) as u8;
        let imm = (opcode & 0xFF) as u8;
        
        return match op {
            0 => Instruction::MovImm { rd, imm },  // 00100 = MOV
            1 => Instruction::CmpImm { rn: rd, imm }, // 00101 = CMP
            2 => Instruction::AddImm8 { rd, imm }, // 00110 = ADD
            3 => Instruction::SubImm8 { rd, imm }, // 00111 = SUB
            _ => Instruction::Unknown(opcode),
        };
    }

    // 2. Add/Sub (Register/Imm3) (T1): 0001 1xx ...
    if (opcode & 0xF800) == 0x1800 {
        let op_sub = (opcode >> 9) & 0x3;
        let rm_imm = ((opcode >> 6) & 0x7) as u8;
        let rn = ((opcode >> 3) & 0x7) as u8;
        let rd = (opcode & 0x7) as u8;
        
        return match op_sub {
            0 => Instruction::AddReg { rd, rn, rm: rm_imm },
            1 => Instruction::SubReg { rd, rn, rm: rm_imm },
            2 => Instruction::AddImm3 { rd, rn, imm: rm_imm },
            3 => Instruction::SubImm3 { rd, rn, imm: rm_imm },
            _ => unreachable!(),
        };
    }
    
    // 3. ALU Operations (T1): 0100 00xx ...
    if (opcode & 0xFC00) == 0x4000 {
        let op_alu = (opcode >> 6) & 0xF;
        let rm = ((opcode >> 3) & 0x7) as u8;
        let rd = (opcode & 0x7) as u8;
        
        return match op_alu {
            0x0 => Instruction::And { rd, rm }, // AND
            0x1 => Instruction::Eor { rd, rm }, // EOR
            0xC => Instruction::Orr { rd, rm }, // ORR
            0xF => Instruction::Mvn { rd, rm }, // MVN
            _ => Instruction::Unknown(opcode), 
        };
    }
    
    // 3.1 BX (T1): 0100 0111 0xxxx 000 (0x4700 mask 0xFF80)
    // Bit 3-6 is Rm.
    if (opcode & 0xFF80) == 0x4700 {
         let rm = ((opcode >> 3) & 0xF) as u8;
         return Instruction::Bx { rm };
    }
    
    // 4. Load/Store (Imm5) (T1): 0110 0... -> STR, 0110 1... -> LDR
    // Format: 0110 Liii iinn nttt
    if (opcode & 0xF000) == 0x6000 {
        let is_load = (opcode & 0x0800) != 0;
        let imm5 = ((opcode >> 6) & 0x1F) as u8;
        // The immediate is scaled by 4 for word access
        let imm = imm5 << 2;
        let rn = ((opcode >> 3) & 0x7) as u8;
        let rt = (opcode & 0x7) as u8;
        
        if is_load {
             return Instruction::LdrImm { rt, rn, imm }; // 0x68xx
        } else {
             return Instruction::StrImm { rt, rn, imm }; // 0x60xx
        }
    }
    
    // 4.1 LDR Literal (T1): 0100 1ttt iiii iiii
    if (opcode & 0xF800) == 0x4800 {
        let rt = ((opcode >> 8) & 0x7) as u8;
        let imm = (opcode & 0xFF) as u8; // imm8 * 4
        return Instruction::LdrLit { rt, imm: imm << 2 };
    }
    
    // 4.2 PUSH/POP
    // PUSH: 1011 010M rrrr rrrr (0xB400)
    if (opcode & 0xFE00) == 0xB400 {
        let m = (opcode & 0x0100) != 0; // LR saved?
        let registers = (opcode & 0xFF) as u8;
        return Instruction::Push { registers, m };
    }
    // POP: 1011 110P rrrr rrrr (0xBC00)
    if (opcode & 0xFE00) == 0xBC00 {
        let p = (opcode & 0x0100) != 0; // PC restored?
        let registers = (opcode & 0xFF) as u8;
        return Instruction::Pop { registers, p };
    }

    // 5. Branch (T1/T2)
    // Unconditional Branch T2: 1110 0...
    if (opcode & 0xF800) == 0xE000 {
        let mut offset = (opcode & 0x7FF) as i32;
        if (offset & 0x400) != 0 {
            offset |= !0x7FF;
        }
        return Instruction::Branch { offset: offset << 1 };
    }
    
    // 6. BL Prefix (Part of 32-bit BL)
    // 1111 0... (0xF000 mask 0xF800) -> BL first half?
    // Actually, T1 BL is two 16-bit instructions.
    // 1111 0sss ssss ssss
    if (opcode & 0xF800) == 0xF000 {
         return Instruction::BlPrefix(opcode);
    }
    // BL Suffix: 1101 x... ? No, BL is F0xx ... F8xx
    // Wait for the CPU step to handle 32-bit reassembly or just return prefix for now.
    
    // NOP: 1011 1111 0000 0000 -> 0xBF00
    if opcode == 0xBF00 {
        return Instruction::Nop;
    }

    Instruction::Unknown(opcode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_mov_cmp_add_sub_imm8() {
        // MOV R0, #42 -> 0x202A
        assert_eq!(decode_thumb_16(0x202A), Instruction::MovImm { rd: 0, imm: 42 });
        // CMP R1, #10 -> 0x290A (0010 1001 0000 1010)
        assert_eq!(decode_thumb_16(0x290A), Instruction::CmpImm { rn: 1, imm: 10 });
        // ADD R2, #5 -> 0x3205
        assert_eq!(decode_thumb_16(0x3205), Instruction::AddImm8 { rd: 2, imm: 5 });
        // SUB R3, #1 -> 0x3B01
        assert_eq!(decode_thumb_16(0x3B01), Instruction::SubImm8 { rd: 3, imm: 1 });
    }

    #[test]
    fn test_decode_add_sub_reg_imm3() {
        // ADD R0, R1, R2 -> 0x1888 (0001 100 0 10 001 000)
        assert_eq!(decode_thumb_16(0x1888), Instruction::AddReg { rd: 0, rn: 1, rm: 2 });
        // SUB R3, R4, R5 -> 0x1B63 (0001 101 1 01 100 011) ?
        // 0001 101 101 100 011 -> 0x1B63
        // Op=1 (SubReg), Rm=5, Rn=4, Rd=3
        assert_eq!(decode_thumb_16(0x1B63), Instruction::SubReg { rd: 3, rn: 4, rm: 5 });
        
        // ADD R1, R2, #7 -> 0x1DD1 (0001 110 111 010 001)
        assert_eq!(decode_thumb_16(0x1DD1), Instruction::AddImm3 { rd: 1, rn: 2, imm: 7 });
        // SUB R0, R0, #1 -> 0x1E40 (0001 111 001 000 000)
        assert_eq!(decode_thumb_16(0x1E40), Instruction::SubImm3 { rd: 0, rn: 0, imm: 1 });
    }

    #[test]
    fn test_decode_ldr_str() {
        // STR R0, [R1, #4] -> 0x6048
        // 0110 0 00001 001 000
        // L=0, imm5=1 (so imm=4), Rn=1, Rt=0
        assert_eq!(decode_thumb_16(0x6048), Instruction::StrImm { rt: 0, rn: 1, imm: 4 });
        
        // LDR R2, [R3, #0] -> 0x681A
        // 0110 1 00000 011 010
        // L=1, imm5=0, Rn=3, Rt=2
        assert_eq!(decode_thumb_16(0x681A), Instruction::LdrImm { rt: 2, rn: 3, imm: 0 });
    }

    #[test]
    fn test_decode_alu() {
        // AND R0, R1 -> 0x4008 (0100 00 0000 001 000)
        assert_eq!(decode_thumb_16(0x4008), Instruction::And { rd: 0, rm: 1 });
        // ORR R2, R3 -> 0x431A (0100 00 1100 011 010)
        assert_eq!(decode_thumb_16(0x431A), Instruction::Orr { rd: 2, rm: 3 });
        // EOR R4, R5 -> 0x406C (0100 00 0001 101 100)
        assert_eq!(decode_thumb_16(0x406C), Instruction::Eor { rd: 4, rm: 5 });
        // MVN R6, R7 -> 0x43FE (0100 00 1111 111 110)
        assert_eq!(decode_thumb_16(0x43FE), Instruction::Mvn { rd: 6, rm: 7 });
    }

    #[test]
    fn test_decode_stack_control() {
        // PUSH {R0, LR} -> 0xB501 (1011 0101 0000 0001)
        // M=1, Regs=0x01
        assert_eq!(decode_thumb_16(0xB501), Instruction::Push { registers: 1, m: true });
        
        // POP {R1, PC} -> 0xBD02 (1011 1101 0000 0010)
        // P=1, Regs=0x02
        assert_eq!(decode_thumb_16(0xBD02), Instruction::Pop { registers: 2, p: true });
        
        // BX R14 -> 0x4770 (0100 0111 0111 0000)
        // Rm=14 (LR)
        assert_eq!(decode_thumb_16(0x4770), Instruction::Bx { rm: 14 });
        
        // LDR R0, [PC, #4] -> 0x4801 (0100 1000 0000 0001)
        // Rt=0, imm=1 (scaled to 4)
        assert_eq!(decode_thumb_16(0x4801), Instruction::LdrLit { rt: 0, imm: 4 });
    }
    
    // Existing tests...
    #[test]
    fn test_decode_nop() {
        assert_eq!(decode_thumb_16(0xBF00), Instruction::Nop);
    }
    #[test]
    fn test_decode_branch() {
         assert_eq!(decode_thumb_16(0xE002), Instruction::Branch { offset: 4 });
    }
}
