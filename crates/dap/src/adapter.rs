use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use labwired_core::{Machine, DebugControl, StopReason, cpu::CortexM};
use labwired_loader::SymbolProvider;

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
        machine.load_firmware(&image).map_err(|e| anyhow!("Failed to load firmware: {:?}", e))?;
        
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
            let reason = machine.step_single().map_err(|e| anyhow!("Step failed: {:?}", e))?;
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
            let reason = machine.run(Some(100_000)).map_err(|e| anyhow!("Run failed: {:?}", e))?;
            Ok(reason)
        } else {
            Err(anyhow!("Machine not initialized"))
        }
    }

    pub fn set_breakpoints(&self, _path: String, _lines: Vec<i64>) -> Result<()> {
        // Map lines to addresses? 
        // For MVP, we only support function addresses or raw addresses.
        // If VS Code sends source breakpoints, we need DWARF.
        // For now, let's assume we can't map lines without DWARF.
        // But we promised "PC-based".
        // DAP client sends `setBreakpoints` with source file and lines.
        // We will need DWARF to support this PROPERLY.
        //
        // Workaround for MVP:
        // User sets breakpoint at "main". VS Code resolves "main" to address if it has symbol info?
        // No, VS Code relies on debug adapter for resolution usually.
        //
        // Alternative: Use `functionBreakpoints`.
        // We will assume the user uses Function Breakpoints in VS Code, which sends function names.
        // We then resolve symbol -> address using symbol table from ELF (which loader has!).
        
        Ok(())
    }
}
