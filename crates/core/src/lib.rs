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

/// Trait for observing simulation events in a modular way.
pub trait SimulationObserver: std::fmt::Debug + Send + Sync {
    fn on_simulation_start(&self) {}
    fn on_simulation_stop(&self) {}
    fn on_step_start(&self, _pc: u32, _opcode: u16) {}
    fn on_step_end(&self, _cycles: u32) {}
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
}

/// Trait representing a memory-mapped peripheral
pub trait Peripheral: std::fmt::Debug + Send {
    fn read(&self, offset: u64) -> SimResult<u8>;
    fn write(&mut self, offset: u64, value: u8) -> SimResult<()>;
    fn tick(&mut self) -> bool {
        false
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

pub struct Machine<C: Cpu> {
    pub cpu: C,
    pub bus: bus::SystemBus,
    pub observers: Vec<Arc<dyn SimulationObserver>>,
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
        let interrupts = self.bus.tick_peripherals();
        for irq in interrupts {
            self.cpu.set_exception_pending(irq);
            tracing::debug!("Exception {} Pend", irq);
        }

        res
    }
}
