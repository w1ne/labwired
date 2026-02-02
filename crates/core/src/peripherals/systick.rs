use crate::SimResult;

/// Mocked SysTick Timer peripheral
/// Standard address: 0xE000_E010
#[derive(Debug, Default)]
pub struct Systick {
    csr: u32,
    rvr: u32,
    cvr: u32,
    calib: u32,
}

impl Systick {
    pub fn new() -> Self {
        Self {
            csr: 0,
            rvr: 0,
            cvr: 0,
            calib: 0x4000_0000, // No reference clock, no skew
        }
    }

    pub fn read(&self, offset: u64) -> SimResult<u32> {
        match offset {
            0x00 => Ok(self.csr),
            0x04 => Ok(self.rvr),
            0x08 => Ok(self.cvr),
            0x0C => Ok(self.calib),
            _ => Ok(0),
        }
    }

    pub fn write(&mut self, offset: u64, value: u32) -> SimResult<()> {
        match offset {
            0x00 => {
                // CSR: Only ENABLE (bit 0), TICKINT (bit 1), CLKSOURCE (bit 2) are writable
                self.csr = value & 0x7;
            }
            0x04 => {
                // RVR: 24-bit reload value
                self.rvr = value & 0x00FF_FFFF;
            }
            0x08 => {
                // CVR: Write clears value
                self.cvr = 0;
                // Clearing CVR also clears COUNTFLAG in CSR? (simplified for now)
                self.csr &= !0x10000;
            }
            _ => {}
        }
        Ok(())
    }

    /// Advance the timer by one tick
    pub fn tick(&mut self) -> bool {
        if (self.csr & 0x1) == 0 {
            return false; // Not enabled
        }

        if self.cvr == 0 {
            self.cvr = self.rvr;
            // Set COUNTFLAG
            self.csr |= 0x10000;
            // Return true if interrupt requested
            return (self.csr & 0x2) != 0;
        } else {
            self.cvr -= 1;
            return false;
        }
    }
}
