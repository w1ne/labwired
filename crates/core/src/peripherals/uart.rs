use crate::SimResult;
use std::io::{self, Write};

/// Simple UART mock.
/// Writes to Data Register (offset 0x0) correspond to stdout writes.
#[derive(Debug, Default)]
pub struct Uart {}

impl Uart {
    pub fn new() -> Self {
        Self::default()
    }
}

impl crate::Peripheral for Uart {
    fn read(&self, offset: u64) -> SimResult<u8> {
        match offset {
            0x04 => Ok(0x01), // TX Ready (bit 0)
            _ => Ok(0),
        }
    }

    fn write(&mut self, offset: u64, value: u8) -> SimResult<()> {
        if offset == 0x00 {
            // Write to Data Register -> Stdout
            print!("{}", value as char);
            io::stdout().flush().unwrap();
        }
        Ok(())
    }
}
