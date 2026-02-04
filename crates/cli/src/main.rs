use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use tracing::info;

const EXIT_PASS: u8 = 0;
const EXIT_ASSERT_FAIL: u8 = 1;
const EXIT_CONFIG_ERROR: u8 = 2;
const EXIT_RUNTIME_ERROR: u8 = 3;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "LabWired Simulator",
    long_about = None,
    subcommand_negates_reqs = true
)]
struct Cli {
    /// Path to the firmware ELF file
    #[arg(short, long)]
    firmware: Option<PathBuf>,

    /// Path to the system manifest (YAML)
    #[arg(short, long)]
    system: Option<PathBuf>,

    /// Enable instruction-level execution tracing
    #[arg(short, long, global = true)]
    trace: bool,

    /// Maximum number of steps to execute (default: 20000)
    #[arg(long, default_value = "20000")]
    max_steps: usize,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deterministic, CI-friendly runner mode driven by a test script (YAML).
    Test(TestArgs),
}

#[derive(Parser, Debug)]
struct TestArgs {
    /// Path to the firmware ELF file
    #[arg(short = 'f', long)]
    firmware: PathBuf,

    /// Path to the system manifest (YAML)
    #[arg(short = 's', long)]
    system: Option<PathBuf>,

    /// Path to the test script (YAML)
    #[arg(short = 'c', long)]
    script: PathBuf,

    /// Override max steps (takes precedence over script)
    #[arg(long)]
    max_steps: Option<usize>,

    /// Disable UART stdout echo (still captured for assertions/artifacts)
    #[arg(long)]
    no_uart_stdout: bool,
}

#[derive(Debug, Deserialize)]
struct TestScript {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default)]
    max_steps: Option<usize>,
    #[serde(default)]
    assertions: Vec<Assertion>,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Assertion {
    UartContains { uart_contains: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Initialize tracing with appropriate level based on --trace flag
    if cli.trace {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    match cli.command {
        Some(Commands::Test(args)) => run_test(args),
        None => run_interactive(cli),
    }
}

fn run_interactive(cli: Cli) -> ExitCode {
    info!("Starting LabWired Simulator");

    let Some(firmware) = cli.firmware else {
        tracing::error!("Missing required --firmware argument");
        return ExitCode::from(EXIT_CONFIG_ERROR);
    };

    let bus = match build_bus(cli.system) {
        Ok(bus) => bus,
        Err(e) => {
            tracing::error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    info!("Loading firmware: {:?}", firmware);
    let program = match labwired_loader::load_elf(&firmware) {
        Ok(program) => program,
        Err(e) => {
            tracing::error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    info!("Firmware Loaded Successfully!");
    info!("Entry Point: {:#x}", program.entry_point);

    let metrics = std::sync::Arc::new(labwired_core::metrics::PerformanceMetrics::new());

    let mut machine = labwired_core::Machine::<labwired_core::cpu::CortexM>::with_bus(bus);
    machine.observers.push(metrics.clone());
    if let Err(e) = machine.load_firmware(&program) {
        tracing::error!("Failed to load firmware into memory: {}", e);
        return ExitCode::from(EXIT_RUNTIME_ERROR);
    }

    info!("Starting Simulation...");
    info!(
        "Initial PC: {:#x}, SP: {:#x}",
        machine.cpu.pc, machine.cpu.sp
    );

    // Run for specified number of steps
    info!("Running for {} steps...", cli.max_steps);
    for step in 0..cli.max_steps {
        match machine.step() {
            Ok(_) => {
                // Periodically report IPS if not in trace mode
                if !cli.trace && step > 0 && step % 10000 == 0 {
                    info!("Progress: {} steps, current IPS: {:.2}", step, metrics.get_ips());
                }
            }
            Err(e) => {
                info!("Simulation Error at step {}: {}", step, e);
                break;
            }
        }
    }

    info!("Simulation loop finished (demo).");
    info!("Final PC: {:#x}", machine.cpu.pc);
    info!("Total Instructions: {}", metrics.get_instructions());
    info!("Total Cycles: {}", metrics.get_cycles());
    info!("Average IPS: {:.2}", metrics.get_ips());

    ExitCode::from(EXIT_PASS)
}

fn run_test(args: TestArgs) -> ExitCode {
    let script = match load_test_script(&args.script) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    if script.schema_version != 1 {
        tracing::error!(
            "Unsupported script schema_version {} (expected 1)",
            script.schema_version
        );
        return ExitCode::from(EXIT_CONFIG_ERROR);
    }

    let max_steps = args
        .max_steps
        .or(script.max_steps)
        .unwrap_or(20000);

    let mut bus = match build_bus(args.system) {
        Ok(bus) => bus,
        Err(e) => {
            tracing::error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let uart_tx = Arc::new(Mutex::new(Vec::new()));
    bus.attach_uart_tx_sink(uart_tx.clone(), !args.no_uart_stdout);

    let program = match labwired_loader::load_elf(&args.firmware) {
        Ok(program) => program,
        Err(e) => {
            tracing::error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let mut machine = labwired_core::Machine::<labwired_core::cpu::CortexM>::with_bus(bus);
    if let Err(e) = machine.load_firmware(&program) {
        tracing::error!("Simulation error during load/reset: {}", e);
        return ExitCode::from(EXIT_RUNTIME_ERROR);
    }

    for step in 0..max_steps {
        if let Err(e) = machine.step() {
            tracing::error!("Simulation error at step {}: {}", step, e);
            return ExitCode::from(EXIT_RUNTIME_ERROR);
        }
    }

    let uart_text = {
        let bytes = uart_tx.lock().map(|g| g.clone()).unwrap_or_default();
        String::from_utf8_lossy(&bytes).to_string()
    };

    for assertion in script.assertions {
        match assertion {
            Assertion::UartContains { uart_contains } => {
                if !uart_text.contains(&uart_contains) {
                    tracing::error!(
                        "Assertion failed: uart_contains {:?} (captured len={})",
                        uart_contains,
                        uart_text.len()
                    );
                    return ExitCode::from(EXIT_ASSERT_FAIL);
                }
            }
        }
    }

    ExitCode::from(EXIT_PASS)
}

fn load_test_script(path: &PathBuf) -> anyhow::Result<TestScript> {
    let f = std::fs::File::open(path)?;
    let script: TestScript = serde_yaml::from_reader(f)?;
    Ok(script)
}

fn build_bus(system_path: Option<PathBuf>) -> anyhow::Result<labwired_core::bus::SystemBus> {
    let bus = if let Some(sys_path) = system_path {
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

    Ok(bus)
}
