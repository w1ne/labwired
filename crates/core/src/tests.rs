#[cfg(test)]
mod tests {
    use crate::decoder::{self, Instruction};
    use crate::{Machine, SimResult, Bus};

    #[test]
    fn test_decoder_mov() {
        // 0x202A => MOV R0, #42
        // 0010 0000 0010 1010
        let instr = decoder::decode_thumb_16(0x202A);
        assert_eq!(instr, Instruction::MovImm { rd: 0, imm: 42 });
    }

    #[test]
    fn test_cpu_execute_mov() {
        let mut machine = Machine::new();
        // Use RAM address because Flash via Bus is read-only
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        // Write opcode to memory
        // 0x202A -> Little Endian: 2A 20
        machine.bus.write_u8(base_addr, 0x2A).unwrap();
        machine.bus.write_u8(base_addr + 1, 0x20).unwrap();
        
        // Step
        machine.step().unwrap();
        
        assert_eq!(machine.cpu.r0, 42);
        assert_eq!(machine.cpu.pc, (base_addr + 2) as u32);
    }

    #[test]
    fn test_cpu_execute_branch() {
        let mut machine = Machine::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        // Unconditional Branch: B <offset>
        // We want to skip over a NOP.
        // 0x2000_0000: B +4 (Offset=2 instructions -> +4 bytes)
        // 0x2000_0002: NOP (Skipped)
        // 0x2000_0004: Target
        
        // Encoding for B +2 (instructions): 0xE002
        // Little Endian: 02 E0
        machine.bus.write_u8(base_addr, 0x02).unwrap();
        machine.bus.write_u8(base_addr+1, 0xE0).unwrap();
        
        // Step
        machine.step().unwrap();
        
        // Expected PC: Base + 4 + (2<<1) = Base + 8
        // Wait, my decoder test says:
        // Encoding: i=2 -> 0xE002
        // Target = PC + 4 + (2 << 1) = PC + 8
        // So valid target is 0x2000_0008.
        
        assert_eq!(machine.cpu.pc, (base_addr + 8) as u32);
    }
}
