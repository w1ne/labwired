use crate::memory::LinearMemory;
use crate::peripherals::nvic::NvicState;
use crate::{Bus, Peripheral, SimResult, SimulationError};
use labwired_config::{parse_size, ChipDescriptor, SystemManifest};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct PeripheralEntry {
    pub name: String,
    pub base: u64,
    pub size: u64,
    pub irq: Option<u32>,
    pub dev: Box<dyn Peripheral>,
}

pub struct SystemBus {
    pub flash: LinearMemory,
    pub ram: LinearMemory,
    pub peripherals: Vec<PeripheralEntry>,
    pub nvic: Option<Arc<NvicState>>,
}

impl Default for SystemBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemBus {
    pub fn new() -> Self {
        // Default initialization for tests
        Self {
            flash: LinearMemory::new(1024 * 1024, 0x0),
            ram: LinearMemory::new(1024 * 1024, 0x2000_0000),
            peripherals: vec![
                PeripheralEntry {
                    name: "systick".to_string(),
                    base: 0xE000_E010,
                    size: 0x10,
                    irq: Some(15),
                    dev: Box::new(crate::peripherals::systick::Systick::new()),
                },
                PeripheralEntry {
                    name: "uart1".to_string(),
                    base: 0x4000_C000,
                    size: 0x1000,
                    irq: None,
                    dev: Box::new(crate::peripherals::uart::Uart::new()),
                },
                PeripheralEntry {
                    name: "gpioa".to_string(),
                    base: 0x4001_0800,
                    size: 0x400,
                    irq: None,
                    dev: Box::new(crate::peripherals::gpio::GpioPort::new()),
                },
                PeripheralEntry {
                    name: "gpiob".to_string(),
                    base: 0x4001_0C00,
                    size: 0x400,
                    irq: None,
                    dev: Box::new(crate::peripherals::gpio::GpioPort::new()),
                },
                PeripheralEntry {
                    name: "gpioc".to_string(),
                    base: 0x4001_1000,
                    size: 0x400,
                    irq: None,
                    dev: Box::new(crate::peripherals::gpio::GpioPort::new()),
                },
                PeripheralEntry {
                    name: "rcc".to_string(),
                    base: 0x4002_1000,
                    size: 0x400,
                    irq: None,
                    dev: Box::new(crate::peripherals::rcc::Rcc::new()),
                },
                PeripheralEntry {
                    name: "tim2".to_string(),
                    base: 0x4000_0000,
                    size: 0x400,
                    irq: Some(28),
                    dev: Box::new(crate::peripherals::timer::Timer::new()),
                },
                PeripheralEntry {
                    name: "tim3".to_string(),
                    base: 0x4000_0400,
                    size: 0x400,
                    irq: Some(29),
                    dev: Box::new(crate::peripherals::timer::Timer::new()),
                },
                PeripheralEntry {
                    name: "i2c1".to_string(),
                    base: 0x4000_5400,
                    size: 0x400,
                    irq: Some(31),
                    dev: Box::new(crate::peripherals::i2c::I2c::new()),
                },
                PeripheralEntry {
                    name: "i2c2".to_string(),
                    base: 0x4000_5800,
                    size: 0x400,
                    irq: Some(33),
                    dev: Box::new(crate::peripherals::i2c::I2c::new()),
                },
                PeripheralEntry {
                    name: "spi1".to_string(),
                    base: 0x4001_3000,
                    size: 0x400,
                    irq: Some(35),
                    dev: Box::new(crate::peripherals::spi::Spi::new()),
                },
                PeripheralEntry {
                    name: "spi2".to_string(),
                    base: 0x4000_3800,
                    size: 0x400,
                    irq: Some(36),
                    dev: Box::new(crate::peripherals::spi::Spi::new()),
                },
            ],
            nvic: None,
        }
    }

    pub fn from_config(chip: &ChipDescriptor, _manifest: &SystemManifest) -> anyhow::Result<Self> {
        let flash_size = parse_size(&chip.flash.size)?;
        let ram_size = parse_size(&chip.ram.size)?;

        let mut bus = Self {
            flash: LinearMemory::new(flash_size as usize, chip.flash.base),
            ram: LinearMemory::new(ram_size as usize, chip.ram.base),
            peripherals: Vec::new(),
            nvic: None,
        };

        for p_cfg in &chip.peripherals {
            let dev: Box<dyn Peripheral> = match p_cfg.r#type.as_str() {
                "uart" => Box::new(crate::peripherals::uart::Uart::new()),
                "systick" => Box::new(crate::peripherals::systick::Systick::new()),
                "gpio" => Box::new(crate::peripherals::gpio::GpioPort::new()),
                "rcc" => Box::new(crate::peripherals::rcc::Rcc::new()),
                "timer" => Box::new(crate::peripherals::timer::Timer::new()),
                "i2c" => Box::new(crate::peripherals::i2c::I2c::new()),
                "spi" => Box::new(crate::peripherals::spi::Spi::new()),
                other => {
                    tracing::warn!(
                        "Unsupported peripheral type '{}' for id '{}'; skipping",
                        other,
                        p_cfg.id
                    );
                    continue;
                }
            };

            let mut dev = dev;
            for ext in &_manifest.external_devices {
                if ext.connection == p_cfg.id {
                    tracing::info!("Stubbing {} on {}", ext.id, p_cfg.id);
                    // For now, if it's a stub, we replace it or wrap it?
                    // Let's replace with StubPeripheral for demonstration
                    dev = Box::new(crate::peripherals::stub::StubPeripheral::new(0x42));
                }
            }

            // Map interrupt numbers based on ID for now (simplified)
            let irq = if p_cfg.id == "systick" {
                Some(15)
            } else {
                None
            };

            bus.peripherals.push(PeripheralEntry {
                name: p_cfg.id.clone(),
                base: p_cfg.base_address,
                size: 0x1000, // Default 4KB page
                irq,
                dev,
            });
        }

        Ok(bus)
    }

    pub fn read_u32(&self, addr: u64) -> SimResult<u32> {
        let b0 = self.read_u8(addr)? as u32;
        let b1 = self.read_u8(addr + 1)? as u32;
        let b2 = self.read_u8(addr + 2)? as u32;
        let b3 = self.read_u8(addr + 3)? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    pub fn write_u32(&mut self, addr: u64, value: u32) -> SimResult<()> {
        self.write_u8(addr, (value & 0xFF) as u8)?;
        self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8)?;
        self.write_u8(addr + 2, ((value >> 16) & 0xFF) as u8)?;
        self.write_u8(addr + 3, ((value >> 24) & 0xFF) as u8)?;
        Ok(())
    }

    pub fn read_u16(&self, addr: u64) -> SimResult<u16> {
        let b0 = self.read_u8(addr)? as u16;
        let b1 = self.read_u8(addr + 1)? as u16;
        Ok(b0 | (b1 << 8))
    }

    pub fn write_u16(&mut self, addr: u64, value: u16) -> SimResult<()> {
        self.write_u8(addr, (value & 0xFF) as u8)?;
        self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8)?;
        Ok(())
    }
}

impl crate::Bus for SystemBus {
    fn read_u8(&self, addr: u64) -> SimResult<u8> {
        if let Some(val) = self.ram.read_u8(addr) {
            return Ok(val);
        }
        if let Some(val) = self.flash.read_u8(addr) {
            return Ok(val);
        }

        // Dynamic Peripherals
        for p in &self.peripherals {
            if addr >= p.base && addr < p.base + p.size {
                return p.dev.read(addr - p.base);
            }
        }

        Err(SimulationError::MemoryViolation(addr))
    }

    fn write_u8(&mut self, addr: u64, value: u8) -> SimResult<()> {
        if self.ram.write_u8(addr, value) {
            return Ok(());
        }
        if self.flash.write_u8(addr, value) {
            return Ok(());
        }

        // Dynamic Peripherals
        for p in &mut self.peripherals {
            if addr >= p.base && addr < p.base + p.size {
                return p.dev.write(addr - p.base, value);
            }
        }

        Err(SimulationError::MemoryViolation(addr))
    }

    fn tick_peripherals(&mut self) -> Vec<u32> {
        let mut interrupts = Vec::new();

        // 1. Collect IRQs from peripherals and pend them in NVIC
        for p in &mut self.peripherals {
            if p.dev.tick() {
                if let Some(irq) = p.irq {
                    if irq >= 16 {
                        if let Some(nvic) = &self.nvic {
                            let idx = ((irq - 16) / 32) as usize;
                            let bit = (irq - 16) % 32;
                            if idx < 8 {
                                nvic.ispr[idx].fetch_or(1 << bit, Ordering::SeqCst);
                            }
                        } else {
                            // No NVIC, pend legacy style
                            interrupts.push(irq);
                        }
                    } else {
                        // Core exceptions bypass NVIC ISPR/ISER
                        interrupts.push(irq);
                    }
                }
            }
        }

        // 2. Scan NVIC for all Pending & Enabled interrupts
        if let Some(nvic) = &self.nvic {
            for idx in 0..8 {
                let mask =
                    nvic.iser[idx].load(Ordering::SeqCst) & nvic.ispr[idx].load(Ordering::SeqCst);
                if mask != 0 {
                    for bit in 0..32 {
                        if (mask & (1 << bit)) != 0 {
                            let irq = 16 + (idx as u32 * 32) + bit;
                            tracing::info!("Bus: NVIC Signaling Pending IRQ {}", irq);
                            interrupts.push(irq);
                        }
                    }
                }
            }
        }

        interrupts
    }
}
