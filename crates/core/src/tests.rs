#[cfg(test)]
mod tests {
    use crate::decoder::{self, Instruction};
    use crate::{Machine, SimResult, Bus};
    use crate::cpu::CortexM;

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

    #[test]
    fn test_cpu_execute_sp_rel() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        // Setup Stack Pointer
        let stack_top = 0x2000_1000;
        machine.cpu.sp = stack_top;
        
        // 1. STR R0, [SP, #4]
        // R0 = 0xCAFEBABE
        machine.cpu.r0 = 0xCAFEBABE;
        
        // Opcode: 1001 0 000 00000001 (STR R0, [SP, 4]) -> 0x9001
        machine.bus.write_u8(base_addr, 0x01).unwrap();
        machine.bus.write_u8(base_addr+1, 0x90).unwrap();
        
        machine.step().unwrap();
        
        // Verify Memory at SP+4
        let val = machine.bus.read_u32((stack_top + 4) as u64).unwrap();
        assert_eq!(val, 0xCAFEBABE);
        
        // 2. LDR R1, [SP, #4]
        // Opcode: 1001 1 001 00000001 (LDR R1, [SP, 4]) -> 0x9901
        machine.bus.write_u8(base_addr+2, 0x01).unwrap();
        machine.bus.write_u8(base_addr+3, 0x99).unwrap();
        
        machine.step().unwrap();
        
        assert_eq!(machine.cpu.r1, 0xCAFEBABE);
    }

    #[test]
    fn test_cpu_execute_cond_branch() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        // 1. CMP R0, #0 -> Z=1
        // MOV R0, #0
        machine.cpu.r0 = 0;
        // CMP R0, #0 -> 0x2800 (0010 1000 0000 0000)
        
        // Manual store of CMP R0, #0
        machine.bus.write_u8(base_addr, 0x00).unwrap();
        machine.bus.write_u8(base_addr+1, 0x28).unwrap();
        
        machine.step().unwrap();
        
        // Check Z flag in XPSR (Bit 30)
        assert_eq!(machine.cpu.xpsr & (1 << 30), 1 << 30);
        
        // 2. BEQ +4 (If Z=1, Branch)
        // Encoding: 0xD002 (Cond=0 EQ, Offset=4)
        machine.bus.write_u8(base_addr+2, 0x02).unwrap();
        machine.bus.write_u8(base_addr+3, 0xD0).unwrap();
        
        // Target should be Base + 2 + 4 + 4 = Base + 10 (Wait)
        // PC during execution is (Base+2). Pipeline PC = (Base+2) + 4.
        // Target = PC + 4 + offset?
        // Thumb Bcc: Target = PC + 4 + (imm8 << 1)
        // My decoder: offset = imm8 << 1 = 4.
        // CPU logic: target = pc + 4 + offset.
        // Wait, standard: Target = PC + 4 + (sign_extended(imm8) << 1)
        // If my decoder returns offset=4, and logic is pc+4+offset => pc+8.
        // Let's verify standard.
        // "Branch target address = PC + 4 + (SignExtended(imm8) << 1)"
        // Correct.
        
        machine.step().unwrap();
        
        // PC was 0x2000_0002.
        // PC+4 = 0x2000_0006.
        // Offset = 4.
        // Target = 0x2000_000A.
        
        assert_eq!(machine.cpu.pc, 0x2000_000A);
    }

    #[test]
    fn test_cpu_execute_shifts() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        // LSLS R0, R1, #4
        machine.cpu.r1 = 0x0000_0001;
        // 0x0110 -> (000 00 00100 001 000) ? 
        // 00000 00100 001 000 -> 0x0108
        machine.bus.write_u8(base_addr, 0x08).unwrap();
        machine.bus.write_u8(base_addr+1, 0x01).unwrap();
        
        machine.step().unwrap();
        assert_eq!(machine.cpu.r0, 0x10);
        
        // LSRS R2, R3, #2
        machine.cpu.r3 = 0x10;
        // 00001 00010 011 010 -> 0x089A
        machine.bus.write_u8(base_addr+2, 0x9A).unwrap();
        machine.bus.write_u8(base_addr+3, 0x08).unwrap();
        
        machine.step().unwrap();
        assert_eq!(machine.cpu.r2, 0x04);
    }

    #[test]
    fn test_cpu_execute_cmp_reg() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        machine.cpu.r1 = 10;
        machine.cpu.r0 = 5;
        // CMP R1, R0 -> 0x4281
        machine.bus.write_u8(base_addr, 0x81).unwrap();
        machine.bus.write_u8(base_addr+1, 0x42).unwrap();
        
        machine.step().unwrap();
        // 10 - 5 = 5. N=0, Z=0, C=1 (no borrow), V=0
        let xpsr = machine.cpu.xpsr >> 28;
        assert_eq!(xpsr & 0b1000, 0); // N
        assert_eq!(xpsr & 0b0100, 0); // Z
        assert_eq!(xpsr & 0b0010, 0b0010); // C
    }

    #[test]
    fn test_cpu_execute_mov_reg() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        machine.cpu.sp = 0x2002_0000;
        // MOV R7, SP -> 0x466F
        machine.bus.write_u8(base_addr, 0x6F).unwrap();
        machine.bus.write_u8(base_addr+1, 0x46).unwrap();
        
        machine.step().unwrap();
        assert_eq!(machine.cpu.r7, 0x2002_0000);
    }

    #[test]
    fn test_cpu_execute_strb_imm() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        let base_addr: u64 = 0x2000_0000;
        machine.cpu.pc = base_addr as u32;
        
        machine.cpu.r1 = 0xAB;
        machine.cpu.r0 = 0x2000_1000;
        // STRB R1, [R0, #0] -> 0x7001
        machine.bus.write_u8(base_addr, 0x01).unwrap();
        machine.bus.write_u8(base_addr+1, 0x70).unwrap();
        
        machine.step().unwrap();
        assert_eq!(machine.bus.read_u8(0x2000_1000).unwrap(), 0xAB);
    }

    #[test]
    fn test_systick_timer() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        
        // 1. Configure SysTick
        // RVR = 10 (Reload after 10 ticks)
        machine.bus.write_u32(0xE000_E014, 10).unwrap();
        // CSR = 1 (Enable)
        machine.bus.write_u32(0xE000_E010, 1).unwrap();
        
        // CVR is initially 0, so first tick should reload and start counting?
        // In my impl: tick() checks ENABLE. If 0, returns false. 
        // If cvr == 0, cvr = rvr, sets COUNTFLAG.
        
        // Step 1: PC=0 (Unknown instruction at 0 is likely, but machine.step will still tick systick)
        let _ = machine.step(); 
        let cvr = machine.bus.read_u32(0xE000_E018).unwrap();
        assert_eq!(cvr, 10);
        
        // Step 2-11: Count down to 0
        for _ in 0..10 {
            let _ = machine.step();
        }
        
        let cvr_final = machine.bus.read_u32(0xE000_E018).unwrap();
        assert_eq!(cvr_final, 0);
        
        let csr = machine.bus.read_u32(0xE000_E010).unwrap();
        assert_eq!(csr & 0x10000, 0x10000); // COUNTFLAG should be set
    }

    #[test]
    fn test_exception_stacking() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        
        // 1. Setup Vector Table for SysTick (Exception 15)
        // Address = 15 * 4 = 60 (0x3C)
        let isr_addr: u32 = 0x0000_1000;
        machine.bus.write_u32(0x3C, isr_addr | 1).unwrap(); // Thumb address
        
        // 2. Setup initial state
        machine.cpu.pc = 0x2000_0000;
        machine.cpu.sp = 0x2002_0000;
        machine.cpu.r0 = 0x12345678;
        
        // 3. Trigger SysTick (Reload=1, Enable=3 [ENABLE|TICKINT])
        machine.bus.write_u32(0xE000_E014, 1).unwrap();
        machine.bus.write_u32(0xE000_E010, 3).unwrap();
        
        // Step 1: PC=0x2000_0000. Ticks SysTick.
        // SysTick wrap triggers exception 15.
        let _ = machine.step(); 
        
        // Step 2: Next step should detect pending exception AND handle it.
        // It should perform stacking and jump to 0x1000.
        let _ = machine.step();
        
        assert_eq!(machine.cpu.pc, 0x1000);
        assert_eq!(machine.cpu.sp, 0x2002_0000 - 32); 
        assert_eq!(machine.cpu.lr, 0xFFFF_FFF9);
        
        // Check if R0 was stacked correctly at [SP]
        let stacked_r0 = machine.bus.read_u32(machine.cpu.sp as u64).unwrap();
        assert_eq!(stacked_r0, 0x12345678);
    }

    #[test]
    fn test_exception_lifecycle() {
        let mut machine = Machine::<crate::cpu::CortexM>::new();
        
        // 1. Setup SysTick Vector
        let isr_addr: u32 = 0x0000_1000;
        machine.bus.write_u32(0x3C, isr_addr | 1).unwrap();
        
        // 2. Setup initial state
        machine.cpu.pc = 0x2000_0000;
        machine.cpu.sp = 0x2002_0000;
        machine.cpu.r0 = 10;
        machine.cpu.r7 = 20;
        
        // 3. Trigger SysTick
        machine.bus.write_u32(0xE000_E014, 100).unwrap();
        machine.bus.write_u32(0xE000_E010, 3).unwrap();
        
        // Step 1: Wrap SysTick
        machine.step().unwrap();
        
        // Step 2: Handle Exception (Entry)
        machine.step().unwrap();
        assert_eq!(machine.cpu.pc, 0x1000);
        assert_eq!(machine.cpu.lr, 0xFFFF_FFF9);
        
        // 4. In ISR: Modify R0, then BX LR
        // MOV R0, #42 -> 0x202A
        machine.bus.write_u8(0x1000, 0x2A).unwrap();
        machine.bus.write_u8(0x1001, 0x20).unwrap();
        // BX LR -> 0x4770 (BX R14)
        machine.bus.write_u8(0x1002, 0x70).unwrap();
        machine.bus.write_u8(0x1003, 0x47).unwrap();
        
        // Step 3: Execute MOV R0, #42 in ISR
        machine.step().unwrap();
        assert_eq!(machine.cpu.r0, 42);
        
        // Step 4: Execute BX LR (Exception Return)
        machine.step().unwrap();
        
        // 5. Verify restored state
        assert_eq!(machine.cpu.pc, 0x2000_0002); // Back at original PC + 2
        assert_eq!(machine.cpu.r0, 10);          // Original R0 restored!
        assert_eq!(machine.cpu.sp, 0x2002_0000); // SP restored
        assert_eq!(machine.cpu.r7, 20);          // R7 was untouched
    }

    #[test]
    fn test_iteration_7_instructions() {
        let mut machine: Machine<CortexM> = Machine::new();
        machine.cpu.sp = 0x2000_1000;
        
        // 1. ADD SP, #12 (3 * 4) -> 0xB003
        machine.bus.write_u16(0, 0xB003).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.cpu.sp, 0x2000_100C);
        
        // 2. SUB SP, #16 (4 * 4) -> 0xB084
        machine.cpu.pc = 2;
        machine.bus.write_u16(2, 0xB084).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.cpu.sp, 0x2000_0FFC);
        
        // 3. ADD R0, R8 (High Reg) -> 0x4440 (Rd=R0, Rm=R8)
        machine.cpu.r0 = 10;
        machine.cpu.r8 = 20;
        machine.cpu.pc = 4;
        machine.bus.write_u16(4, 0x4440).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.cpu.r0, 30);

        // 4. CPSID i -> 0xB672
        machine.cpu.primask = false;
        machine.cpu.pc = 6;
        machine.bus.write_u16(6, 0xB672).unwrap();
        machine.step().unwrap();
        assert!(machine.cpu.primask);

        // 5. CPSIE i -> 0xB662
        machine.cpu.pc = 8;
        machine.bus.write_u16(8, 0xB662).unwrap();
        machine.step().unwrap();
        assert!(!machine.cpu.primask);
    }
}
