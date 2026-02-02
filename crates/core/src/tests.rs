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
        let mut machine = Machine::<crate::cpu::CortexM>::new();
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
        let mut machine = Machine::<crate::cpu::CortexM>::new();
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
    #[test]
    fn test_cpu_execute_ldr_str() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        // 1. STR R0, [R1, #0]
        // R0 = 0xDEADBEEF
        // R1 = 0x2000_0010 (Target RAM)
        machine.cpu.r0 = 0xDEADBEEF;
        machine.cpu.r1 = 0x2000_0010;
        
        // Opcode STR R0, [R1, #0] -> 0x6008
        // 0110 0 00000 001 000
        machine.bus.write_u8(base_addr, 0x08).unwrap();
        machine.bus.write_u8(base_addr+1, 0x60).unwrap();
        
        machine.step().unwrap();
        
        // precise verify RAM
        let val = machine.bus.read_u32(0x2000_0010).unwrap();
        assert_eq!(val, 0xDEADBEEF);
        
        // 2. LDR R2, [R1, #0]
        // Should load 0xDEADBEEF into R2
        // Opcode LDR R2, [R1, #0] -> 0x680A
        // 0110 1 00000 001 010
        machine.bus.write_u8(base_addr+2, 0x0A).unwrap();
        machine.bus.write_u8(base_addr+3, 0x68).unwrap();
        
        machine.step().unwrap();
        
        assert_eq!(machine.cpu.r2, 0xDEADBEEF);
    }

    #[test]
    fn test_uart_write() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        // Base PC = RAM
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        // Code:
        // MOV R0, #72 ('H')
        // STR R0, [R1] (where R1 points to UART)
        
        // Manual setup for simplicity
        machine.cpu.r0 = 72; // 'H'
        machine.cpu.r1 = 0x4000_C000;
        
        // STR R0, [R1, #0] -> 0x6008
        // 0110 0 00000 001 000
        machine.bus.write_u8(base_addr, 0x08).unwrap();
        machine.bus.write_u8(base_addr+1, 0x60).unwrap();
        
        // Capture stdout? Rust test harness captures it.
        // We mainly verify it doesn't crash.
        // Ideally we would mock stdout, but for this level of sim, 
        // ensuring it runs without MemoryViolation is enough.
        machine.step().unwrap();
    }
}
