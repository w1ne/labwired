pub mod bus;
pub mod cpu;
pub mod decoder;
pub mod interrupt;
pub mod memory;
pub mod metrics;
pub mod multi_core;
pub mod peripherals;
pub mod signals;

use std::any::Any;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

mod tests;

#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("Memory access violation at {0:#x}")]
    MemoryViolation(u64),
    #[error("Instruction decoding error at {0:#x}")]
    DecodeError(u64),
}

pub type SimResult<T> = Result<T, SimulationError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeripheralTickResult {
    pub irq: bool,
    pub cycles: u32,
}

/// Trait for observing simulation events in a modular way.
pub trait SimulationObserver: std::fmt::Debug + Send + Sync {
    fn on_simulation_start(&self) {}
    fn on_simulation_stop(&self) {}
    fn on_step_start(&self, _pc: u32, _opcode: u16) {}
    fn on_step_end(&self, _cycles: u32) {}
    fn on_peripheral_tick(&self, _name: &str, _cycles: u32) {}
}

/// Trait representing a CPU architecture
pub trait Cpu {
    fn reset(&mut self);
    fn step(
        &mut self,
        bus: &mut dyn Bus,
        observers: &[Arc<dyn SimulationObserver>],
    ) -> SimResult<()>;
    fn set_pc(&mut self, val: u32);
    fn get_pc(&self) -> u32;
    fn set_sp(&mut self, val: u32);
    fn set_exception_pending(&mut self, exception_num: u32);
    fn get_vtor(&self) -> u32;
    fn set_vtor(&mut self, val: u32);
    fn set_shared_vtor(&mut self, vtor: Arc<AtomicU32>);

    // Debug Access
    fn get_register(&self, id: u8) -> u32;
    fn set_register(&mut self, id: u8, val: u32);
}

/// Trait representing a memory-mapped peripheral
pub trait Peripheral: std::fmt::Debug + Send {
    fn read(&self, offset: u64) -> SimResult<u8>;
    fn write(&mut self, offset: u64, value: u8) -> SimResult<()>;
    fn tick(&mut self) -> PeripheralTickResult {
        PeripheralTickResult {
            irq: false,
            cycles: 0,
        }
    }
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }
}

/// Trait representing the system bus
pub trait Bus {
    fn read_u8(&self, addr: u64) -> SimResult<u8>;
    fn write_u8(&mut self, addr: u64, value: u8) -> SimResult<()>;
    fn tick_peripherals(&mut self) -> Vec<u32>; // Returns list of pending exception numbers

    fn read_u16(&self, addr: u64) -> SimResult<u16> {
        let b0 = self.read_u8(addr)? as u16;
        let b1 = self.read_u8(addr + 1)? as u16;
        // Little Endian
        Ok(b0 | (b1 << 8))
    }

    fn read_u32(&self, addr: u64) -> SimResult<u32> {
        let b0 = self.read_u8(addr)? as u32;
        let b1 = self.read_u8(addr + 1)? as u32;
        let b2 = self.read_u8(addr + 2)? as u32;
        let b3 = self.read_u8(addr + 3)? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    fn write_u32(&mut self, addr: u64, value: u32) -> SimResult<()> {
        self.write_u8(addr, (value & 0xFF) as u8)?;
        self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8)?;
        self.write_u8(addr + 2, ((value >> 16) & 0xFF) as u8)?;
        self.write_u8(addr + 3, ((value >> 24) & 0xFF) as u8)?;
        Ok(())
    }

    fn write_u16(&mut self, addr: u64, value: u16) -> SimResult<()> {
        self.write_u8(addr, (value & 0xFF) as u8)?;
        self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8)?;
        Ok(())
    }
}

use std::collections::HashSet;

/// Trait for controlling the machine in debug mode
pub trait DebugControl {
    fn add_breakpoint(&mut self, addr: u32);
    fn remove_breakpoint(&mut self, addr: u32);
    fn clear_breakpoints(&mut self);

    /// Run until breakpoint or steps limit
    fn run(&mut self, max_steps: Option<u32>) -> SimResult<StopReason>;

    /// Step a single instruction
    fn step_single(&mut self) -> SimResult<StopReason>;

    fn read_core_reg(&self, id: u8) -> u32;
    fn write_core_reg(&mut self, id: u8, val: u32);

    fn read_memory(&self, addr: u32, len: usize) -> SimResult<Vec<u8>>;
    fn write_memory(&mut self, addr: u32, data: &[u8]) -> SimResult<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    Breakpoint(u32),
    StepDone,
    MaxStepsReached,
    ManualStop,
}

pub struct Machine<C: Cpu> {
    pub cpu: C,
    pub bus: bus::SystemBus,
    pub observers: Vec<Arc<dyn SimulationObserver>>,

    // Debug state
    pub breakpoints: HashSet<u32>,
}

impl<C: Cpu + Default> Machine<C> {
    pub fn new() -> Self {
        Self::with_bus(bus::SystemBus::new())
    }

    /// Construct a machine around an existing bus, and ensure core system peripherals
    /// (SCB + NVIC) are installed and wired up (shared VTOR + NVIC pending state).
    pub fn with_bus(mut bus: bus::SystemBus) -> Self {
        let vtor = Arc::new(AtomicU32::new(0));
        let nvic_state = Arc::new(peripherals::nvic::NvicState::default());

        let mut cpu = C::default();
        cpu.set_shared_vtor(vtor.clone());

        bus.nvic = Some(nvic_state.clone());

        // Ensure SCB exists (VTOR relocation)
        let scb = peripherals::scb::Scb::new(vtor);
        if let Some(p) = bus
            .peripherals
            .iter_mut()
            .find(|p| p.name == "scb" || p.base == 0xE000_ED00)
        {
            p.name = "scb".to_string();
            p.base = 0xE000_ED00;
            p.size = 0x40;
            p.irq = None;
            p.dev = Box::new(scb);
        } else {
            bus.peripherals.push(bus::PeripheralEntry {
                name: "scb".to_string(),
                base: 0xE000_ED00,
                size: 0x40,
                irq: None,
                dev: Box::new(scb),
            });
        }

        // Ensure NVIC exists (shared pending/enabled state)
        let nvic = peripherals::nvic::Nvic::new(nvic_state);
        if let Some(p) = bus
            .peripherals
            .iter_mut()
            .find(|p| p.name == "nvic" || p.base == 0xE000_E100)
        {
            p.name = "nvic".to_string();
            p.base = 0xE000_E100;
            p.size = 0x400;
            p.irq = None;
            p.dev = Box::new(nvic);
        } else {
            bus.peripherals.push(bus::PeripheralEntry {
                name: "nvic".to_string(),
                base: 0xE000_E100,
                size: 0x400,
                irq: None,
                dev: Box::new(nvic),
            });
        }

        Self {
            cpu,
            bus,
            observers: Vec::new(),
            breakpoints: HashSet::new(),
        }
    }
}

impl<C: Cpu + Default> Default for Machine<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Cpu> Machine<C> {
    pub fn load_firmware(&mut self, image: &memory::ProgramImage) -> SimResult<()> {
        for segment in &image.segments {
            // Try loading into Flash first
            if !self.bus.flash.load_from_segment(segment) {
                // If not flash, try RAM? Or just warn?
                // For now, let's assume everything goes to Flash or RAM mapped spaces
                if !self.bus.ram.load_from_segment(segment) {
                    tracing::warn!(
                        "Failed to load segment at {:#x} - outside of memory map",
                        segment.start_addr
                    );
                }
            }
        }

        for observer in &self.observers {
            observer.on_simulation_start();
        }
        self.reset()?;

        // Fallback if vector table is missing/zero
        if self.cpu.get_pc() == 0 {
            self.cpu.set_pc(image.entry_point as u32);
        }

        Ok(())
    }

    pub fn reset(&mut self) -> SimResult<()> {
        self.cpu.reset();

        let vtor = self.cpu.get_vtor() as u64;
        if let Ok(sp) = self.bus.read_u32(vtor) {
            self.cpu.set_sp(sp);
        }
        if let Ok(pc) = self.bus.read_u32(vtor + 4) {
            self.cpu.set_pc(pc);
        }

        Ok(())
    }

    pub fn step(&mut self) -> SimResult<()> {
        let res = self.cpu.step(&mut self.bus, &self.observers);

        // Propagate peripherals
        let (interrupts, costs) = self.bus.tick_peripherals_with_costs();
        for c in costs {
            if let Some(p) = self.bus.peripherals.get(c.index) {
                for observer in &self.observers {
                    observer.on_peripheral_tick(&p.name, c.cycles);
                }
            }
        }
        for irq in interrupts {
            self.cpu.set_exception_pending(irq);
            tracing::debug!("Exception {} Pend", irq);
        }

        res
    }
}

impl<C: Cpu> DebugControl for Machine<C> {
    fn add_breakpoint(&mut self, addr: u32) {
        self.breakpoints.insert(addr);
    }

    fn remove_breakpoint(&mut self, addr: u32) {
        self.breakpoints.remove(&addr);
    }

    fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    fn run(&mut self, max_steps: Option<u32>) -> SimResult<StopReason> {
        let mut steps = 0;
        loop {
            // Check breakpoints BEFORE stepping
            let pc = self.cpu.get_pc();
            // Note: breakpoints typically match the exact PC.
            // Thumb instructions are at even addresses, usually.
            // If the user sets a BP at an odd address (Thumb function pointer), we should mask it?
            // Usually DAP clients send the symbol address.
            // Let's assume exact match for now, but mask LSB.
            let pc_aligned = pc & !1;

            if self.breakpoints.contains(&pc_aligned) {
                return Ok(StopReason::Breakpoint(pc));
            }

            self.step()?;
            steps += 1;

            if let Some(max) = max_steps {
                if steps >= max {
                    return Ok(StopReason::MaxStepsReached);
                }
            }
        }
    }

    fn step_single(&mut self) -> SimResult<StopReason> {
        self.step()?;
        Ok(StopReason::StepDone)
    }

    fn read_core_reg(&self, id: u8) -> u32 {
        self.cpu.get_register(id)
    }

    fn write_core_reg(&mut self, id: u8, val: u32) {
        self.cpu.set_register(id, val);
    }

    fn read_memory(&self, addr: u32, len: usize) -> SimResult<Vec<u8>> {
        let mut data = Vec::with_capacity(len);
        for i in 0..len {
            let byte = self.bus.read_u8((addr as u64) + (i as u64))?;
            data.push(byte);
        }
        Ok(data)
    }

    fn write_memory(&mut self, addr: u32, data: &[u8]) -> SimResult<()> {
        for (i, byte) in data.iter().enumerate() {
            self.bus.write_u8((addr as u64) + (i as u64), *byte)?;
        }
        Ok(())
    }
}
