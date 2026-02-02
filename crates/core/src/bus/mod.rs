use crate::memory::LinearMemory;
use crate::{SimResult, SimulationError, Peripheral, Bus};
use labwired_config::{ChipDescriptor, SystemManifest, parse_size};

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
                    size: 0x100,
                    irq: Some(15),
                    dev: Box::new(crate::peripherals::systick::Systick::new()),
                },
                PeripheralEntry {
                    name: "uart1".to_string(),
                    base: 0x4000_C000,
                    size: 0x1000,
                    irq: None,
                    dev: Box::new(crate::peripherals::uart::Uart::new()),
                }
            ],
        }
    }

    pub fn from_config(chip: &ChipDescriptor, _manifest: &SystemManifest) -> anyhow::Result<Self> {
        let flash_size = parse_size(&chip.flash.size)?;
        let ram_size = parse_size(&chip.ram.size)?;
        
        let mut bus = Self {
            flash: LinearMemory::new(flash_size as usize, chip.flash.base),
            ram: LinearMemory::new(ram_size as usize, chip.ram.base),
            peripherals: Vec::new(),
        };

        for p_cfg in &chip.peripherals {
            let mut dev: Box<dyn Peripheral> = match p_cfg.r#type.as_str() {
                "uart" => Box::new(crate::peripherals::uart::Uart::new()),
                "systick" => Box::new(crate::peripherals::systick::Systick::new()),
                _ => continue, // Unsupported for now
            };
            
            // Apply external device stubs if connected to this peripheral ID
            for ext in &_manifest.external_devices {
                if ext.connection == p_cfg.id {
                    tracing::info!("Stubbing {} on {}", ext.id, p_cfg.id);
                    // For now, if it's a stub, we replace it or wrap it?
                    // Let's replace with StubPeripheral for demonstration
                    dev = Box::new(crate::peripherals::stub::StubPeripheral::new(0x42));
                }
            }

            // Map interrupt numbers based on ID for now (simplified)
            let irq = if p_cfg.id == "systick" { Some(15) } else { None };

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
        for p in &mut self.peripherals {
            if p.dev.tick() {
                if let Some(irq) = p.irq {
                    interrupts.push(irq);
                }
            }
        }
        interrupts
    }
}
