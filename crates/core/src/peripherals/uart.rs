// LabWired - Firmware Simulation Platform
// Copyright (C) 2026 Andrii Shylenko
//
// This software is released under the MIT License.
// See the LICENSE file in the project root for full license information.

use crate::SimResult;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Simple UART mock.
/// Writes to Data Register (offset 0x0) correspond to stdout writes.
#[derive(Debug, Default, serde::Serialize)]
pub struct Uart {
    #[serde(skip)]
    sink: Option<Arc<Mutex<Vec<u8>>>>,
    echo_stdout: bool,
}

impl Uart {
    pub fn new() -> Self {
        Self {
            sink: None,
            echo_stdout: true,
        }
    }

    pub fn set_sink(&mut self, sink: Option<Arc<Mutex<Vec<u8>>>>, echo_stdout: bool) {
        self.sink = sink;
        self.echo_stdout = echo_stdout;
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
            if let Some(sink) = &self.sink {
                if let Ok(mut guard) = sink.lock() {
                    guard.push(value);
                }
            }

            if self.echo_stdout {
                // Write to Data Register -> Stdout
                print!("{}", value as char);
                io::stdout().flush().unwrap();
            }
        }
        Ok(())
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn snapshot(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}
