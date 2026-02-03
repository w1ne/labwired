use clap::Parser;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the firmware ELF file
    #[arg(short, long)]
    firmware: PathBuf,

    /// Path to the system manifest (YAML)
    #[arg(short, long)]
    system: Option<PathBuf>,

    /// Enable instruction-level execution tracing
    #[arg(short, long)]
    trace: bool,

    /// Maximum number of steps to execute (default: 20000)
    #[arg(long, default_value = "20000")]
    max_steps: usize,
}

fn main() -> anyhow::Result<()> {
    // Initialize tracing with appropriate level based on --trace flag
    let args = Args::parse();

    if args.trace {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    info!("Starting LabWired Simulator");

    let bus = if let Some(sys_path) = args.system {
        info!("Loading system manifest: {:?}", sys_path);
        let manifest = labwired_config::SystemManifest::from_file(&sys_path)?;
        let chip_path = sys_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(&manifest.chip);
        info!("Loading chip descriptor: {:?}", chip_path);
        let chip = labwired_config::ChipDescriptor::from_file(&chip_path)?;
        labwired_core::bus::SystemBus::from_config(&chip, &manifest)?
    } else {
        info!("Using default hardware configuration");
        labwired_core::bus::SystemBus::new()
    };

    info!("Loading firmware: {:?}", args.firmware);
    let program = labwired_loader::load_elf(&args.firmware)?;

    info!("Firmware Loaded Successfully!");
    info!("Entry Point: {:#x}", program.entry_point);

    let mut machine = labwired_core::Machine {
        cpu: labwired_core::cpu::CortexM::default(),
        bus,
    };
    machine
        .load_firmware(&program)
        .expect("Failed to load firmware into memory");

    info!("Starting Simulation...");
    info!(
        "Initial PC: {:#x}, SP: {:#x}",
        machine.cpu.pc, machine.cpu.sp
    );

    // Run for specified number of steps
    info!("Running for {} steps...", args.max_steps);
    for step in 0..args.max_steps {
        match machine.step() {
            Ok(_) => {
                // trace logged in step
            }
            Err(e) => {
                info!("Simulation Error at step {}: {}", step, e);
                break;
            }
        }
    }

    info!("Simulation loop finished (demo).");
    info!("Final PC: {:#x}", machine.cpu.pc);

    Ok(())
}
