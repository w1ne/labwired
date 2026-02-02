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
}
