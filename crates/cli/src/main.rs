use clap::Parser;
use tracing::info;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the firmware ELF file
    #[arg(short, long)]
    firmware: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    info!("Starting LabWired Simulator");
    info!("Loading firmware: {:?}", args.firmware);

    let program = labwired_loader::load_elf(&args.firmware)?;
    
    info!("Firmware Loaded Successfully!");
    info!("Entry Point: {:#x}", program.entry_point);
    
    let mut machine = labwired_core::Machine::new();
    machine.load_firmware(&program).expect("Failed to load firmware into memory");
    
    info!("Starting Simulation...");
    info!("Initial PC: {:#x}, SP: {:#x}", machine.cpu.pc, machine.cpu.sp);
    
    // Run for 10 steps as a demo
    for i in 0..10 {
        match machine.step() {
            Ok(_) => {
                // trace logged in step
            },
            Err(e) => {
                info!("Simulation Error at step {}: {}", i, e);
                break;
            }
        }
    }
    
    info!("Simulation loop finished (demo).");
    info!("Final PC: {:#x}", machine.cpu.pc);
    
    Ok(())
}
