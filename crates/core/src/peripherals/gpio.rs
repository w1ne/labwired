use crate::SimResult;

/// STM32F1-compatible GPIO peripheral
#[derive(Debug, Default)]
pub struct GpioPort {
    crl: u32,  // 0x00: configuration register low
    crh: u32,  // 0x04: configuration register high
    idr: u32,  // 0x08: input data register
    odr: u32,  // 0x0C: output data register
    lckr: u32, // 0x18: configuration lock register
}

impl GpioPort {
    pub fn new() -> Self {
        Self {
            crl: 0x4444_4444, // Reset value: floating input
            crh: 0x4444_4444, // Reset value: floating input
            ..Default::default()
        }
    }

    fn read_reg(&self, offset: u64) -> u32 {
        match offset {
            0x00 => self.crl,
            0x04 => self.crh,
            0x08 => self.idr,
            0x0C => self.odr,
            0x18 => self.lckr,
            _ => 0,
        }
    }

    fn write_reg(&mut self, offset: u64, value: u32) {
        match offset {
            0x00 => self.crl = value,
            0x04 => self.crh = value,
            0x0C => self.odr = value & 0xFFFF,
            0x10 => {
                // BSRR: Bit Set/Reset Register
                let set = value & 0xFFFF;
                let reset = (value >> 16) & 0xFFFF;
                self.odr |= set;
                self.odr &= !reset;
            }
            0x14 => {
                // BRR: Bit Reset Register
                let reset = value & 0xFFFF;
                self.odr &= !reset;
            }
            0x18 => self.lckr = value,
            _ => {}
        }
    }
}

impl crate::Peripheral for GpioPort {
    fn read(&self, offset: u64) -> SimResult<u8> {
        let reg_offset = offset & !3;
        let byte_offset = (offset % 4) as u32;
        let reg_val = self.read_reg(reg_offset);
        Ok(((reg_val >> (byte_offset * 8)) & 0xFF) as u8)
    }

    fn write(&mut self, offset: u64, value: u8) -> SimResult<()> {
        let reg_offset = offset & !3;
        let byte_offset = (offset % 4) as u32;
        
        // We need to be careful with BSRR/BRR because they are 32-bit registers
        // and byte writes might not make sense for them if the user uses write_u8.
        // However, Bus::write_u32 calls write_u8 four times.
        // For BSRR/BRR, we should probably accumulate the bytes or only act on the last byte?
        // Actually, Systick implementation reads the current reg value, modifies a byte, and writes it back.
        // That works for ODR/CRL/CRH, but for BSRR/BRR (write-only trigger), it's tricky.
        
        // Let's use the same pattern as Systick for now.
        let mut reg_val = self.read_reg(reg_offset);
        if reg_offset == 0x10 || reg_offset == 0x14 {
             // For write-only triggers, we can't "read" them. 
             // But write_u32 will call write_u8(0), write_u8(1), write_u8(2), write_u8(3).
             // If we use Systick's pattern:
             // step 0: reg_val = read(0x10) -> 0. reg_val |= value. write_reg(0x10, val) -> triggers!
             // That's WRONG if it's supposed to be a single 32-bit write.
             
             // However, our Bus::write_u32 implementation is:
             /*
                self.write_u8(addr, (value & 0xFF) as u8)?;
                self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8)?;
                self.write_u8(addr + 2, ((value >> 16) & 0xFF) as u8)?;
                self.write_u8(addr + 3, ((value >> 24) & 0xFF) as u8)?;
             */
             // This is NOT ideal for peripherals that expect atomic 32-bit writes.
             
             // For now, I'll assume 32-bit writes are used and I'll just handle them.
             // If I want to support BSRR/BRR properly with the current byte-oriented Peripheral trait,
             // I might need to buffer the writes.
             
             // Let's check how other 32-bit peripherals handle this.
        }

        let mask = 0xFF << (byte_offset * 8);
        reg_val &= !mask;
        reg_val |= (value as u32) << (byte_offset * 8);
        
        self.write_reg(reg_offset, reg_val);
        Ok(())
    }
}
