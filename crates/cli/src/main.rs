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
    for (i, segment) in program.segments.iter().enumerate() {
        info!("Segment {}: Address={:#x}, Size={} bytes", i, segment.start_addr, segment.data.len());
    }
    
    info!("Simulation not yet implemented. Exiting.");
    
    Ok(())
}
