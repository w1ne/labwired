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
    pub pending_exceptions: u32, // Bitmask
}

impl CortexM {
    pub fn new() -> Self {
        Self::default()
    }

    fn read_reg(&self, n: u8) -> u32 {
        match n {
            0 => self.r0, 1 => self.r1, 2 => self.r2, 3 => self.r3,
            4 => self.r4, 5 => self.r5, 6 => self.r6, 7 => self.r7,
            8 => self.r8, 9 => self.r9, 10 => self.r10, 11 => self.r11,
            12 => self.r12, 13 => self.sp, 14 => self.lr, 15 => self.pc,
            _ => 0,
        }
    }
    
    fn write_reg(&mut self, n: u8, val: u32) {
        match n {
            0 => self.r0 = val, 1 => self.r1 = val, 2 => self.r2 = val, 3 => self.r3 = val,
            4 => self.r4 = val, 5 => self.r5 = val, 6 => self.r6 = val, 7 => self.r7 = val,
            8 => self.r8 = val, 9 => self.r9 = val, 10 => self.r10 = val, 11 => self.r11 = val,
            12 => self.r12 = val, 13 => self.sp = val, 14 => self.lr = val, 15 => self.pc = val,
            _ => {},
        }
    }
    
    fn update_nz(&mut self, result: u32) {
        let n = (result >> 31) & 1;
        let z = if result == 0 { 1 } else { 0 };
        // Clear N/Z (bits 31, 30)
        self.xpsr &= !(0xC000_0000);
        self.xpsr |= (n << 31) | (z << 30);
    }
    
    fn update_nzcv(&mut self, result: u32, carry: bool, overflow: bool) {
        let n = (result >> 31) & 1;
        let z = if result == 0 { 1 } else { 0 };
        let c = if carry { 1 } else { 0 };
        let v = if overflow { 1 } else { 0 };
        
        self.xpsr &= !(0xF000_0000);
        self.xpsr |= (n << 31) | (z << 30) | (c << 29) | (v << 28);
    }

    fn check_condition(&self, cond: u8) -> bool {
        let n = (self.xpsr >> 31) & 1 == 1;
        let z = (self.xpsr >> 30) & 1 == 1;
        let c = (self.xpsr >> 29) & 1 == 1;
        let v = (self.xpsr >> 28) & 1 == 1;
        
        match cond {
            0x0 => z,             // EQ (Equal)
            0x1 => !z,            // NE (Not Equal)
            0x2 => c,             // CS/HS (Carry Set)
            0x3 => !c,            // CC/LO (Carry Clear)
            0x4 => n,             // MI (Minus)
            0x5 => !n,            // PL (Plus)
            0x6 => v,             // VS (Overflow)
            0x7 => !v,            // VC (No Overflow)
            0x8 => c && !z,       // HI (Unsigned Higher)
            0x9 => !c || z,       // LS (Unsigned Lower or Same)
            0xA => n == v,        // GE (Signed Greater or Equal)
            0xB => n != v,        // LT (Signed Less Than)
            0xC => !z && (n == v),// GT (Signed Greater Than)
            0xD => z || (n != v), // LE (Signed Less or Equal)
            0xE => true,          // AL (Always)
            _ => false,           // Undefined/Reserved
        }
    }

    fn branch_to(&mut self, addr: u32, bus: &mut dyn Bus) -> SimResult<()> {
        if (addr & 0xF000_0000) == 0xF000_0000 {
            // EXC_RETURN logic
            self.exception_return(bus)?;
        } else {
            self.pc = addr & !1;
        }
        Ok(())
    }

    fn exception_return(&mut self, bus: &mut dyn Bus) -> SimResult<()> {
        // Perform Unstacking
        let frame_ptr = self.sp;
        
        self.r0 = bus.read_u32(frame_ptr as u64)?;
        self.r1 = bus.read_u32((frame_ptr + 4) as u64)?;
        self.r2 = bus.read_u32((frame_ptr + 8) as u64)?;
        self.r3 = bus.read_u32((frame_ptr + 12) as u64)?;
        self.r12 = bus.read_u32((frame_ptr + 16) as u64)?;
        self.lr = bus.read_u32((frame_ptr + 20) as u64)?;
        self.pc = bus.read_u32((frame_ptr + 24) as u64)?;
        self.xpsr = bus.read_u32((frame_ptr + 28) as u64)?;
        
        self.sp = frame_ptr + 32;
        
        tracing::info!("Exception return to {:#x}", self.pc);
        Ok(())
    }
}

impl Cpu for CortexM {
    fn reset(&mut self) {
        self.pc = 0x0000_0000;
        self.sp = 0x2000_0000;
        self.pending_exceptions = 0;
    }

    fn step(&mut self, bus: &mut dyn Bus) -> SimResult<()> {
        // Check for pending exceptions before executing instruction
        if self.pending_exceptions != 0 {
            // Find highest priority exception (Simplified: highest bit)
            let exception_num = 31 - self.pending_exceptions.leading_zeros();
            self.pending_exceptions &= !(1 << exception_num);
            
            // Perform Stacking (Simplified)
            let sp = self.sp;
            let frame_ptr = sp.wrapping_sub(32);
            
            // Stack: R0, R1, R2, R3, R12, LR, PC, xPSR
            let _ = bus.write_u32(frame_ptr as u64, self.r0);
            let _ = bus.write_u32((frame_ptr + 4) as u64, self.r1);
            let _ = bus.write_u32((frame_ptr + 8) as u64, self.r2);
            let _ = bus.write_u32((frame_ptr + 12) as u64, self.r3);
            let _ = bus.write_u32((frame_ptr + 16) as u64, self.r12);
            let _ = bus.write_u32((frame_ptr + 20) as u64, self.lr);
            let _ = bus.write_u32((frame_ptr + 24) as u64, self.pc);
            let _ = bus.write_u32((frame_ptr + 28) as u64, self.xpsr);
            
            self.sp = frame_ptr;
            
            // EXC_RETURN: Thread Mode, MSP
            self.lr = 0xFFFF_FFF9; 
            
            // Jump to ISR handler
            let vector_addr = exception_num * 4;
            if let Ok(handler) = bus.read_u32(vector_addr as u64) {
                self.pc = handler & !1;
                tracing::info!("Exception {} trigger, jump to {:#x}", exception_num, self.pc);
            }
            
            return Ok(());
        }

        // ... (existing logic)
        // Fetch 16-bit thumb instruction
        let fetch_pc = self.pc & !1;
        let opcode = bus.read_u16(fetch_pc as u64)?;
        
        // Decode
        let instruction = decoder::decode_thumb_16(opcode);
        
        tracing::debug!("PC={:#x}, Opcode={:#04x}, Instr={:?}", self.pc, opcode, instruction);
        
        // Execute
        let mut pc_increment = 2; // Default for 16-bit instruction
        
        match instruction {
            Instruction::Nop => { /* Do nothing */ },
            Instruction::MovImm { rd, imm } => {
                self.write_reg(rd, imm as u32);
                self.update_nz(imm as u32);
            },
            Instruction::Branch { offset } => {
                 let target = (self.pc as i32 + 4 + offset) as u32;
                 self.pc = target;
                 pc_increment = 0;
            },
            // Arithmetic 
            Instruction::AddReg { rd, rn, rm } => {
                let op1 = self.read_reg(rn);
                let op2 = self.read_reg(rm);
                let (res, c, v) = add_with_flags(op1, op2);
                self.write_reg(rd, res);
                self.update_nzcv(res, c, v);
            },
            Instruction::AddImm3 { rd, rn, imm } => {
                let op1 = self.read_reg(rn);
                let (res, c, v) = add_with_flags(op1, imm as u32);
                self.write_reg(rd, res);
                self.update_nzcv(res, c, v);
            },
            Instruction::AddImm8 { rd, imm } => {
                let op1 = self.read_reg(rd);
                let (res, c, v) = add_with_flags(op1, imm as u32);
                self.write_reg(rd, res);
                self.update_nzcv(res, c, v);
            },
            Instruction::SubReg { rd, rn, rm } => {
                let op1 = self.read_reg(rn);
                let op2 = self.read_reg(rm);
                let (res, c, v) = sub_with_flags(op1, op2);
                self.write_reg(rd, res);
                self.update_nzcv(res, c, v);
            },
            Instruction::SubImm3 { rd, rn, imm } => {
                let op1 = self.read_reg(rn);
                let (res, c, v) = sub_with_flags(op1, imm as u32);
                self.write_reg(rd, res);
                self.update_nzcv(res, c, v);
            },
            Instruction::SubImm8 { rd, imm } => {
                let op1 = self.read_reg(rd);
                let (res, c, v) = sub_with_flags(op1, imm as u32);
                self.write_reg(rd, res);
                self.update_nzcv(res, c, v);
            },
            Instruction::CmpImm { rn, imm } => {
                let op1 = self.read_reg(rn);
                let (res, c, v) = sub_with_flags(op1, imm as u32);
                self.update_nzcv(res, c, v);
            },
            Instruction::CmpReg { rn, rm } => {
                let op1 = self.read_reg(rn);
                let op2 = self.read_reg(rm);
                let (res, c, v) = sub_with_flags(op1, op2);
                self.update_nzcv(res, c, v);
            },
            Instruction::MovReg { rd, rm } => {
                let val = self.read_reg(rm);
                self.write_reg(rd, val);
            },
            // Logic
            Instruction::And { rd, rm } => {
                let res = self.read_reg(rd) & self.read_reg(rm);
                self.write_reg(rd, res);
                self.update_nz(res);
            },
            Instruction::Orr { rd, rm } => {
                let res = self.read_reg(rd) | self.read_reg(rm);
                self.write_reg(rd, res);
                self.update_nz(res);
            },
            Instruction::Eor { rd, rm } => {
                let res = self.read_reg(rd) ^ self.read_reg(rm);
                self.write_reg(rd, res);
                self.update_nz(res);
            },
            Instruction::Mvn { rd, rm } => {
                let res = !self.read_reg(rm);
                self.write_reg(rd, res);
                self.update_nz(res);
            },
            
            // Shifts
            Instruction::Lsl { rd, rm, imm } => {
                let val = self.read_reg(rm);
                let res = val.wrapping_shl(imm as u32);
                self.write_reg(rd, res);
                self.update_nz(res);
                // Note: Carry out not fully implemented for shifts yet
            },
            Instruction::Lsr { rd, rm, imm } => {
                let val = self.read_reg(rm);
                let res = if imm == 0 { 0 } else { val.wrapping_shr(imm as u32) };
                // Actually LSR imm=0 is 32 in some contexts, but Thumb T1 usually:
                // imm5=0 for LSL is imm=0. imm5=0 for LSR is imm=32.
                // For MVP, letting wrapping_shr handle basics. 
                self.write_reg(rd, res);
                self.update_nz(res);
            },
            Instruction::Asr { rd, rm, imm } => {
                let val = self.read_reg(rm) as i32;
                let res = (if imm == 0 { val >> 31 } else { val >> (imm as u32) }) as u32;
                self.write_reg(rd, res);
                self.update_nz(res);
            },
            
            // Memory Operations (Word)
            Instruction::LdrImm { rt, rn, imm } => {
                let base = self.read_reg(rn);
                let addr = base.wrapping_add(imm as u32);
                if let Ok(val) = bus.read_u32(addr as u64) {
                    self.write_reg(rt, val);
                } else {
                    tracing::error!("Bus Read Fault at {:#x}", addr);
                }
            },
            Instruction::StrImm { rt, rn, imm } => {
                 let base = self.read_reg(rn);
                 let addr = base.wrapping_add(imm as u32);
                 let val = self.read_reg(rt);
                 if let Err(_) = bus.write_u32(addr as u64, val) {
                     tracing::error!("Bus Write Fault at {:#x}", addr);
                 }
            },
            
            Instruction::LdrLit { rt, imm } => {
                // ... (existing)
                let pc_val = (self.pc & !3) + 4;
                let addr = pc_val.wrapping_add(imm as u32);
                if let Ok(val) = bus.read_u32(addr as u64) {
                    self.write_reg(rt, val);
                } else {
                    tracing::error!("Bus Read Fault (LdrLit) at {:#x}", addr);
                }
            },
            
            Instruction::LdrSp { rt, imm } => {
                let addr = self.sp.wrapping_add(imm as u32);
                if let Ok(val) = bus.read_u32(addr as u64) {
                    self.write_reg(rt, val);
                } else {
                    tracing::error!("Bus Read Fault (LdrSp) at {:#x}", addr);
                }
            },
            Instruction::StrSp { rt, imm } => {
                let addr = self.sp.wrapping_add(imm as u32);
                let val = self.read_reg(rt);
                if let Err(_) = bus.write_u32(addr as u64, val) {
                    tracing::error!("Bus Write Fault (StrSp) at {:#x}", addr);
                }
            },
            
            // Memory Operations (Byte)
            Instruction::LdrbImm { rt, rn, imm } => {
                let base = self.read_reg(rn);
                let addr = base.wrapping_add(imm as u32);
                if let Ok(val) = bus.read_u8(addr as u64) {
                    self.write_reg(rt, val as u32);
                } else {
                    tracing::error!("Bus Read Fault (LDRB) at {:#x}", addr);
                }
            },
            Instruction::StrbImm { rt, rn, imm } => {
                let base = self.read_reg(rn);
                let addr = base.wrapping_add(imm as u32);
                let val = (self.read_reg(rt) & 0xFF) as u8;
                if let Err(_) = bus.write_u8(addr as u64, val) {
                    tracing::error!("Bus Write Fault (STRB) at {:#x}", addr);
                }
            },

            // Stack Operations
            Instruction::Push { registers, m } => {
                let mut sp = self.read_reg(13);
                // Cycle through R14(LR), R7..R0 high to low
                
                // If M (LR) is set, push LR first (highest address)
                if m {
                    sp = sp.wrapping_sub(4);
                    let val = self.read_reg(14);
                    if let Err(_) = bus.write_u32(sp as u64, val) { tracing::error!("Stack Overflow (PUSH LR)"); }
                }
                
                // Registers R7 down to R0
                for i in (0..=7).rev() {
                    if (registers & (1 << i)) != 0 {
                        sp = sp.wrapping_sub(4);
                        let val = self.read_reg(i);
                        if let Err(_) = bus.write_u32(sp as u64, val) { tracing::error!("Stack Overflow (PUSH R{})", i); }
                    }
                }
                
                self.write_reg(13, sp);
            },
            Instruction::Pop { registers, p } => {
                let mut sp = self.read_reg(13);
                
                // Registers R0 up to R7
                for i in 0..=7 {
                    if (registers & (1 << i)) != 0 {
                        if let Ok(val) = bus.read_u32(sp as u64) {
                            self.write_reg(i, val);
                        }
                        sp = sp.wrapping_add(4);
                    }
                }
                
                // If P (PC) is set, pop PC (lowest address?? No, highest)
                // POP is inverse of PUSH. PUSH pushed LR last (lowest addr) ?? 
                // Wait. PUSH stores STMDB (Decrement Before). Highest reg = Highest address.
                // R0 is lowest register. LR is highest.
                // PUSH order: LR, R7, ... R0.
                // Stack grows down. 
                // Low Addr [ R0 | R1 | ... | LR ] High Addr.
                // So POP (LDMIA) should read: R0, ... R7, PC.
                // My PUSH loop: 
                // 1. If LR, sub 4, write LR. (Top of stack, highest addr - 4)
                // 2. Loop 7 down to 0: sub 4, write Rx.
                // Result: R0 is at current SP. LR is at SP + n*4.
                
                // My POP loop:
                // 1. Loop 0 to 7: read, add 4. (Read R0, R1...)
                // 2. If PC, read, add 4.
                
                if p {
                    if let Ok(val) = bus.read_u32(sp as u64) {
                        self.branch_to(val, bus)?;
                        pc_increment = 0; // Branch taken
                    }
                    sp = sp.wrapping_add(4);
                }
                
                self.write_reg(13, sp);
            },
            
            // Control Flow
            Instruction::Bl { offset } => {
                // BL: Branch with Link.
                // LR = Next Instruction Address | 1 (Thumb bit)
                let _next_pc = self.pc + 4; // 32-bit instruction size for BL? 
                // Wait. BL is decoded as 32-bit.
                // If we assume decode_thumb_16 handled a 32-bit stream, then PC increment should be adjusted?
                // Or does `decode_thumb_16` return `BlPrefix` and then we handle it?
                // The current `decoder` returns `Bl` with full offset if it sees the pair??
                // NO. My decoder implementation for BL (in previous turn) was:
                // `Instruction::Bl { offset: offset << 1 }`
                // But `decode_thumb_16` ONLY sees 16 bits. It cannot see the second half!
                // Real decoding of BL requires fetching 32 bits.
                
                // CRITICAL CORRECTION: `decode_thumb_16` is 16-bit.
                // BL is 32-bit (encoded as two 16-bit halves).
                // Fetch loop fetches 16 bits.
                // 1. Fetch High Half (0xF0xx). Returns BlPrefix?
                // 2. Fetch Low Half (0xF8xx). Combine?
                
                // My logic in decoder needs revisit. I put `Bl { offset }` thinking T1/T2 but BL is always 32-bit in Thumb-2.
                // T1 encoding of BL doesn't exist as single 16-bit.
                
                // For now, let's just implement the execution stub assuming the decoder *somehow* gave us the full BL.
                // But since the decoder only sees 16 bits, we need to handle the prefix state in the CPU loop!
                
                self.lr = (self.pc + 4) | 1;
                let target = (self.pc as i32 + 4 + offset) as u32;
                self.pc = target;
                pc_increment = 0;
            },
            Instruction::BranchCond { cond, offset } => {
                if self.check_condition(cond) {
                    let target = (self.pc as i32 + 4 + offset) as u32;
                    self.pc = target;
                    pc_increment = 0;
                }
            },
            Instruction::Bx { rm } => {
                let target = self.read_reg(rm);
                self.branch_to(target, bus)?;
                pc_increment = 0;
            },
            
            Instruction::Prefix32(h1) => {
                // ... (existing)
                // This is the first half of a 32-bit instruction
                let next_pc = (self.pc & !1) + 2;
                if let Ok(h2) = bus.read_u16(next_pc as u64) {
                    if (h2 & 0xD000) == 0xD000 && (h1 & 0xF800) == 0xF000 {
                        // Reassemble BL offset (T1 encoding)
                        let s = ((h1 >> 10) & 0x1) as i32;
                        let imm10 = (h1 & 0x3FF) as i32;
                        let j1 = ((h2 >> 13) & 0x1) as i32;
                        let j2 = ((h2 >> 11) & 0x1) as i32;
                        let imm11 = (h2 & 0x7FF) as i32;
                        
                        let i1 = (!(j1 ^ s)) & 0x1;
                        let i2 = (!(j2 ^ s)) & 0x1;
                        
                        let mut offset = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);
                        // Sign extend from bit 24
                        if (offset & (1 << 24)) != 0 {
                            offset |= !0x01FF_FFFF;
                        }
                        
                        self.lr = (self.pc + 4) | 1;
                        self.pc = (self.pc as i32 + 4 + offset) as u32;
                        pc_increment = 0;
                    } else if (h1 & 0xFBF0) == 0xF240 {
                        // MOVW (T1)
                        let i = (h1 >> 10) & 0x1;
                        let imm4 = h1 & 0xF;
                        let imm3 = (h2 >> 12) & 0x7;
                        let rd = ((h2 >> 8) & 0xF) as u8;
                        let imm8 = h2 & 0xFF;
                        
                        // Encoding: imm4 : i : imm3 : imm8
                        let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
                        self.write_reg(rd, imm16 as u32);
                        pc_increment = 4;
                    } else if (h1 & 0xFBF0) == 0xF2C0 {
                        // MOVT (T1)
                        let i = (h1 >> 10) & 0x1;
                        let imm4 = h1 & 0xF;
                        let imm3 = (h2 >> 12) & 0x7;
                        let rd = ((h2 >> 8) & 0xF) as u8;
                        let imm8 = h2 & 0xFF;
                        
                        // Encoding: imm4 : i : imm3 : imm8
                        let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
                        let old_val = self.read_reg(rd);
                        let new_val = (old_val & 0x0000FFFF) | ((imm16 as u32) << 16);
                        self.write_reg(rd, new_val);
                        pc_increment = 4;
                    } else {
                        tracing::warn!("Unknown 32-bit instruction: {:#06x} {:#06x} at {:#x}", h1, h2, self.pc);
                        pc_increment = 4;
                    }
                } else {
                    tracing::error!("Bus Read Fault (32-bit suffix) at {:#x}", next_pc);
                    pc_increment = 2;
                }
            },
            Instruction::Movw { .. } | Instruction::Movt { .. } => {
                unreachable!("32-bit instructions should be handled in Prefix32 path");
            },
            
            Instruction::Unknown(op) => {
                tracing::warn!("Unknown instruction at {:#x}: Opcode {:#06x}", self.pc, op);
                pc_increment = 2; // Skip 16-bit
            }
        }
        
        self.pc = self.pc.wrapping_add(pc_increment);
        
        Ok(())
    }

    fn set_pc(&mut self, val: u32) {
        self.pc = val;
    }
    
    fn get_pc(&self) -> u32 {
        self.pc
    }
    
    fn set_sp(&mut self, val: u32) {
        self.sp = val;
    }

    fn set_exception_pending(&mut self, exception_num: u32) {
        if exception_num < 32 {
            self.pending_exceptions |= 1 << exception_num;
        }
    }
}

fn add_with_flags(op1: u32, op2: u32) -> (u32, bool, bool) {
    let (res, overflow1) = op1.overflowing_add(op2);
    let carry = overflow1; 
    let neg_op1 = (op1 as i32) < 0;
    let neg_op2 = (op2 as i32) < 0;
    let neg_res = (res as i32) < 0;
    let overflow = (neg_op1 == neg_op2) && (neg_res != neg_op1);
    (res, carry, overflow)
}

fn sub_with_flags(op1: u32, op2: u32) -> (u32, bool, bool) {
    let (res, borrow) = op1.overflowing_sub(op2);
    let carry = !borrow; 
    let neg_op1 = (op1 as i32) < 0;
    let neg_op2 = (op2 as i32) < 0;
    let neg_res = (res as i32) < 0;
    let overflow = (neg_op1 != neg_op2) && (neg_res != neg_op1);
    (res, carry, overflow)
}
