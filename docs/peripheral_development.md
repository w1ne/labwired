# Peripheral Development Guide

This guide explains how to develop custom, decoupled peripheral models for LabWired.

## The Peripheral Trait

All peripherals in LabWired must implement the `Peripheral` trait located in `labwired_core`.

```rust
pub trait Peripheral: std::fmt::Debug + Send {
    /// Read a single byte from the peripheral at the given offset
    fn read(&self, offset: u64) -> SimResult<u8>;

    /// Write a single byte to the peripheral at the given offset
    fn write(&mut self, offset: u64, value: u8) -> SimResult<()>;

    /// Progress the peripheral state by one "tick"
    /// Returns any IRQs generated, cycles consumed, and any DMA bus requests
    fn tick(&mut self) -> PeripheralTickResult {
        PeripheralTickResult::default()
    }

    /// Return a JSON-serializable snapshot of the internal state
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    // Downcasting support for internal communication
    fn as_any(&self) -> Option<&dyn Any> { None }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> { None }
}
```

## Implementation Best Practices

### 1. State Management
Peripherals are generally "dumb" state containers. Use bit manipulation to handle multi-byte registers.

### 2. Multi-byte Access
If your peripheral registers are 32-bit (common in ARM), implement helper methods to handle byte-wise routing in `read` and `write`:

```rust
fn read(&self, offset: u64) -> SimResult<u8> {
    let reg_val = match offset & !3 {
        0x00 => self.state_reg,
        0x04 => self.data_reg,
        _ => 0,
    };
    let byte_shift = (offset % 4) * 8;
    Ok(((reg_val >> byte_shift) & 0xFF) as u8)
}
```

### 3. Ticking & Cycle Accounting
The `tick()` method is called once per simulation step. Use this to simulate:
- Data processing delays
- Interrupt triggers
- Real-time counters

### 4. Snapshots
Always derive `serde::Serialize` on your peripheral struct and implement `snapshot()` to enable state-saving features. Use `#[serde(skip)]` for non-serializable fields like callbacks or `Arc<Mutex<...>>`.

## Example: Simple Temperature Sensor

Below is a complete implementation of a mock I2C-like temperature sensor with a Status Register (SR) and a Data Register (DR).

```rust
use crate::{Peripheral, PeripheralTickResult, SimResult};
use std::any::Any;

#[derive(Debug, serde::Serialize)]
pub struct TempSensor {
    pub sr: u32, // Status: Bit 0 = Busy, Bit 1 = Data Ready
    pub dr: u32, // Data: Temperature in Celsius
    
    #[serde(skip)]
    update_interval: u32,
    #[serde(skip)]
    ticks: u32,
}

impl TempSensor {
    pub fn new(interval: u32) -> Self {
        Self { sr: 0, dr: 25, update_interval: interval, ticks: 0 }
    }
}

impl Peripheral for TempSensor {
    fn read(&self, offset: u64) -> SimResult<u8> {
        let val = match offset {
            0x00..=0x03 => self.sr,
            0x04..=0x07 => self.dr,
            _ => 0,
        };
        let shift = (offset % 4) * 8;
        Ok(((val >> shift) & 0xFF) as u8)
    }

    fn write(&mut self, offset: u64, _value: u8) -> SimResult<()> {
        // Temperature sensor registers are read-only in this mock
        Ok(())
    }

    fn tick(&mut self) -> PeripheralTickResult {
        self.ticks += 1;
        let mut irq = false;
        
        if self.ticks >= self.update_interval {
            self.ticks = 0;
            self.dr += 1; // Simulate rising temperature
            self.sr |= 0x2; // Set Data Ready bit
            irq = true; // Signal interrupt
        }

        PeripheralTickResult { irq, cycles: 1 }
    }

    fn snapshot(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    fn as_any(&self) -> Option<&dyn Any> { Some(self) }
}
```

## DMA Bus Mastering

If your peripheral needs to perform DMA transfers, it can return `DmaRequest`s from `tick()`. 

```rust
impl Peripheral for MyDmaController {
    fn tick(&mut self) -> PeripheralTickResult {
        let mut dma_requests = Vec::new();
        if self.active {
            dma_requests.push(DmaRequest {
                addr: self.src_addr as u64,
                val: 0, // Not used for Read
                direction: DmaDirection::Read,
            });
            dma_requests.push(DmaRequest {
                addr: self.dest_addr as u64,
                val: 0x42, // Value to write
                direction: DmaDirection::Write,
            });
        }
        PeripheralTickResult {
            irq: false,
            cycles: 1,
            dma_requests,
        }
    }
}
```

> [!NOTE]
> The `SystemBus` executes these requests after the peripheral tick phase.

## Integrating Your Peripheral

To use your peripheral:
1. Register it in the `crates/core/src/peripherals/mod.rs`.
2. Map it in your `SystemBus` configuration.
3. (Optional) Define it in a YAML chip descriptor for dynamic loading.

## Summary Checklist
- [ ] Implement `read` and `write` with byte-alignment logic.
- [ ] Use `tick()` for time-based behavior and IRQs.
- [ ] Derive `Serialize` and implement `snapshot()`.
- [ ] Add `as_any` for downcasting if high-level API access is needed.
