use anyhow::{anyhow, Result};
use labwired_core::{cpu::CortexM, DebugControl, Machine, StopReason};
use labwired_loader::SymbolProvider;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct LabwiredAdapter {
    pub machine: Arc<Mutex<Option<Machine<CortexM>>>>,
    pub symbols: Arc<Mutex<Option<SymbolProvider>>>,
}

impl LabwiredAdapter {
    pub fn new() -> Self {
        Self {
            machine: Arc::new(Mutex::new(None)),
            symbols: Arc::new(Mutex::new(None)),
        }
    }

    pub fn load_firmware(&self, path: PathBuf) -> Result<()> {
        // labwired-loader load_elf takes &Path
        let image = labwired_loader::load_elf(&path)?;

        let mut machine = Machine::<CortexM>::new();
        machine
            .load_firmware(&image)
            .map_err(|e| anyhow!("Failed to load firmware: {:?}", e))?;

        *self.machine.lock().unwrap() = Some(machine);

        // Load symbols
        if let Ok(syms) = SymbolProvider::new(&path) {
            *self.symbols.lock().unwrap() = Some(syms);
        } else {
            tracing::warn!("No debug symbols found or failed to parse: {:?}", path);
        }

        Ok(())
    }

    pub fn lookup_source(&self, addr: u64) -> Option<labwired_loader::SourceLocation> {
        self.symbols.lock().unwrap().as_ref()?.lookup(addr)
    }

    pub fn get_pc(&self) -> Result<u32> {
        let guard = self.machine.lock().unwrap();
        if let Some(machine) = guard.as_ref() {
            Ok(machine.read_core_reg(15)) // PC is R15
        } else {
            Err(anyhow!("Machine not initialized"))
        }
    }

    pub fn get_register(&self, id: u8) -> Result<u32> {
        let guard = self.machine.lock().unwrap();
        if let Some(machine) = guard.as_ref() {
            Ok(machine.read_core_reg(id))
        } else {
            Err(anyhow!("Machine not initialized"))
        }
    }

    pub fn step(&self) -> Result<StopReason> {
        let mut guard = self.machine.lock().unwrap();
        if let Some(machine) = guard.as_mut() {
            let reason = machine
                .step_single()
                .map_err(|e| anyhow!("Step failed: {:?}", e))?;
            Ok(reason)
        } else {
            Err(anyhow!("Machine not initialized"))
        }
    }

    pub fn continue_execution(&self) -> Result<StopReason> {
        let mut guard = self.machine.lock().unwrap();
        if let Some(machine) = guard.as_mut() {
            // Run for a chunk of steps or until breakpoint
            // For interactivity, we might want to run in a loop and release lock?
            // But DAP requests (pause) need to acquire lock.
            // Simplified: Run 1000 steps, check if we should stop?
            // Or just run(). usage of `run(None)` runs forever until breakpoint.
            let reason = machine
                .run(Some(100_000))
                .map_err(|e| anyhow!("Run failed: {:?}", e))?;
            Ok(reason)
        } else {
            Err(anyhow!("Machine not initialized"))
        }
    }

    pub fn set_breakpoints(&self, path: String, lines: Vec<i64>) -> Result<()> {
        let mut addresses = Vec::new();

        let syms_guard = self.symbols.lock().unwrap();
        if let Some(syms) = syms_guard.as_ref() {
            for line in lines {
                if let Some(addr) = syms.location_to_pc(&path, line as u32) {
                    addresses.push(addr as u32);
                } else {
                    tracing::warn!("Could not resolve breakpoint at {}:{}", path, line);
                }
            }
        }

        let mut machine_guard = self.machine.lock().unwrap();
        if let Some(machine) = machine_guard.as_mut() {
            machine.clear_breakpoints();
            for addr in addresses {
                machine.add_breakpoint(addr);
                tracing::info!("Breakpoint set at {:#x}", addr);
            }
        }

        Ok(())
    }

    pub fn read_memory(&self, addr: u64, len: usize) -> Result<Vec<u8>> {
        let machine_guard = self.machine.lock().unwrap();
        if let Some(machine) = machine_guard.as_ref() {
            machine
                .read_memory(addr as u32, len)
                .map_err(|e| anyhow!("Memory read failed: {:?}", e))
        } else {
            Err(anyhow!("Machine not initialized"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_adapter_breakpoints() {
        let elf_path = PathBuf::from("../../target/thumbv7m-none-eabi/debug/firmware");
        if !elf_path.exists() {
            return;
        }

        let adapter = LabwiredAdapter::new();
        adapter
            .load_firmware(elf_path)
            .expect("Failed to load firmware");

        // Set breakpoint at main.rs:11
        adapter
            .set_breakpoints("main.rs".to_string(), vec![11])
            .expect("Failed to set breakpoints");
    }

    #[test]
    fn test_adapter_read_memory() {
        let elf_path = PathBuf::from("../../target/thumbv7m-none-eabi/debug/firmware");
        if !elf_path.exists() {
            return;
        }

        let adapter = LabwiredAdapter::new();
        adapter
            .load_firmware(elf_path)
            .expect("Failed to load firmware");

        // Read first few bytes of Flash (Vector Table)
        let data = adapter.read_memory(0x0, 4).expect("Failed to read memory");
        assert_eq!(data.len(), 4);
    }
}
