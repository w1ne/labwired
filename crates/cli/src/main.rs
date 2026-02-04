use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

use labwired_config::{StopReason, TestAssertion, TestScript};

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
    firmware: Option<PathBuf>,

    /// Path to the system manifest (YAML)
    #[arg(short = 's', long)]
    system: Option<PathBuf>,

    /// Path to the test script (YAML)
    #[arg(short = 'c', long)]
    script: PathBuf,

    /// Override max steps (takes precedence over script)
    #[arg(long)]
    max_steps: Option<u64>,

    /// Disable UART stdout echo (still captured for assertions/artifacts)
    #[arg(long)]
    no_uart_stdout: bool,

    /// Directory to write test artifacts (result.json, uart.log)
    #[arg(long)]
    output_dir: Option<PathBuf>,

    /// Optional path to write a JUnit XML report for CI systems
    #[arg(long)]
    junit: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    status: String,
    steps_executed: u64,
    cycles: u64,
    instructions: u64,
    stop_reason: StopReason,
    assertions: Vec<AssertionResult>,
    firmware_hash: String,
    config: TestConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AssertionResult {
    assertion: TestAssertion,
    passed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestConfig {
    firmware: PathBuf,
    system: Option<PathBuf>,
    script: PathBuf,
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
                    info!(
                        "Progress: {} steps, current IPS: {:.2}",
                        step,
                        metrics.get_ips()
                    );
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
    let script = match TestScript::from_file(&args.script) {
        Ok(s) => s,
        Err(e) => {
            error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let firmware_path = match args
        .firmware
        .clone()
        .or_else(|| Some(resolve_script_path(&args.script, &script.inputs.firmware)))
    {
        Some(p) => p,
        None => {
            error!("Missing firmware path (provide --firmware or set inputs.firmware in script)");
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let system_path = args.system.clone().or_else(|| {
        script
            .inputs
            .system
            .as_deref()
            .map(|s| resolve_script_path(&args.script, s))
    });

    let firmware_bytes = match std::fs::read(&firmware_path) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to read firmware {:?}: {}", firmware_path, e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let max_steps = args.max_steps.unwrap_or(script.limits.max_steps);
    // Guard against accidentally huge runs from CI misconfiguration.
    const MAX_ALLOWED_STEPS: u64 = 50_000_000;
    if max_steps > MAX_ALLOWED_STEPS {
        error!(
            "max_steps {} exceeds MAX_ALLOWED_STEPS {}",
            max_steps, MAX_ALLOWED_STEPS
        );
        return ExitCode::from(EXIT_CONFIG_ERROR);
    }

    let mut bus = match build_bus(system_path.clone()) {
        Ok(bus) => bus,
        Err(e) => {
            error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let uart_tx = Arc::new(Mutex::new(Vec::new()));
    bus.attach_uart_tx_sink(uart_tx.clone(), !args.no_uart_stdout);

    let program = match labwired_loader::load_elf(&firmware_path) {
        Ok(program) => program,
        Err(e) => {
            error!("{:#}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let metrics = std::sync::Arc::new(labwired_core::metrics::PerformanceMetrics::new());
    let mut machine = labwired_core::Machine::<labwired_core::cpu::CortexM>::with_bus(bus);
    machine.observers.push(metrics.clone());

    if let Err(e) = machine.load_firmware(&program) {
        let err_msg = format!("Simulation error during load/reset: {}", e);
        error!("{}", err_msg);
        write_outputs(
            &args,
            "error",
            0,
            &metrics,
            StopReason::Halt,
            vec![],
            &firmware_bytes,
            &uart_tx,
            &firmware_path,
            system_path.as_ref(),
            std::time::Duration::from_secs(0),
        );
        return ExitCode::from(EXIT_RUNTIME_ERROR);
    }

    let start = std::time::Instant::now();
    let mut stop_reason = StopReason::MaxSteps;
    let mut steps_executed: u64 = 0;
    let mut sim_error_happened = false;

    for step in 0..max_steps {
        if let Some(wall_time_ms) = script.limits.wall_time_ms {
            if start.elapsed().as_millis() >= wall_time_ms as u128 {
                stop_reason = StopReason::WallTime;
                break;
            }
        }

        steps_executed = step + 1;
        if let Err(e) = machine.step() {
            sim_error_happened = true;
            stop_reason = match e {
                labwired_core::SimulationError::MemoryViolation(_) => StopReason::MemoryViolation,
                labwired_core::SimulationError::DecodeError(_) => StopReason::DecodeError,
            };
            error!("Simulation error at step {}: {}", step, e);
            break;
        }
    }

    let uart_text = {
        let bytes = uart_tx.lock().map(|g| g.clone()).unwrap_or_default();
        String::from_utf8_lossy(&bytes).to_string()
    };

    let mut assertion_results = Vec::new();
    let mut all_passed = true;
    let mut expected_stop_reason_matched = false;

    for assertion in script.assertions.clone() {
        let passed = match &assertion {
            TestAssertion::UartContains(a) => uart_text.contains(&a.uart_contains),
            TestAssertion::UartRegex(a) => simple_regex_is_match(&a.uart_regex, &uart_text),
            TestAssertion::ExpectedStopReason(a) => a.expected_stop_reason == stop_reason,
        };

        if matches!(assertion, TestAssertion::ExpectedStopReason(_)) && passed {
            expected_stop_reason_matched = true;
        }

        if !passed {
            all_passed = false;
            error!(
                "Assertion failed: {:?} (captured len={})",
                assertion,
                uart_text.len()
            );
        }

        assertion_results.push(AssertionResult { assertion, passed });
    }

    let status = if !all_passed {
        "fail"
    } else if stop_reason == StopReason::WallTime && !expected_stop_reason_matched {
        "fail"
    } else if sim_error_happened && !expected_stop_reason_matched {
        "error"
    } else {
        "pass"
    };

    let duration = start.elapsed();
    write_outputs(
        &args,
        status,
        steps_executed,
        &metrics,
        stop_reason.clone(),
        assertion_results,
        &firmware_bytes,
        &uart_tx,
        &firmware_path,
        system_path.as_ref(),
        duration,
    );

    if !all_passed || (stop_reason == StopReason::WallTime && !expected_stop_reason_matched) {
        ExitCode::from(EXIT_ASSERT_FAIL)
    } else if sim_error_happened && !expected_stop_reason_matched {
        ExitCode::from(EXIT_RUNTIME_ERROR)
    } else {
        ExitCode::from(EXIT_PASS)
    }
}

fn write_outputs(
    args: &TestArgs,
    status: &str,
    steps_executed: u64,
    metrics: &labwired_core::metrics::PerformanceMetrics,
    stop_reason: StopReason,
    assertions: Vec<AssertionResult>,
    firmware_bytes: &[u8],
    uart_tx: &Arc<Mutex<Vec<u8>>>,
    firmware_path: &Path,
    system_path: Option<&PathBuf>,
    duration: std::time::Duration,
) {
    let mut hasher = Sha256::new();
    hasher.update(firmware_bytes);
    let firmware_hash = format!("{:x}", hasher.finalize());

    let assertions_for_junit = assertions.clone();
    let result = TestResult {
        status: status.to_string(),
        steps_executed,
        cycles: metrics.get_cycles(),
        instructions: metrics.get_instructions(),
        stop_reason,
        assertions,
        firmware_hash,
        config: TestConfig {
            firmware: firmware_path.to_path_buf(),
            system: system_path.cloned(),
            script: args.script.clone(),
        },
    };

    if let Some(output_dir) = &args.output_dir {
        if let Err(e) = std::fs::create_dir_all(output_dir) {
            error!("Failed to create output directory {:?}: {}", output_dir, e);
        } else {
            // result.json
            let result_path = output_dir.join("result.json");
            match std::fs::File::create(&result_path) {
                Ok(f) => {
                    if let Err(e) = serde_json::to_writer_pretty(f, &result) {
                        error!("Failed to write result.json: {}", e);
                    }
                }
                Err(e) => error!("Failed to create result.json: {}", e),
            }

            // uart.log
            let uart_path = output_dir.join("uart.log");
            let bytes = uart_tx.lock().map(|g| g.clone()).unwrap_or_default();
            if let Err(e) = std::fs::write(&uart_path, bytes) {
                error!("Failed to write uart.log: {}", e);
            }

            // junit.xml
            let junit_path = output_dir.join("junit.xml");
            if let Err(e) = write_junit_xml(
                &junit_path,
                status,
                duration,
                &result.stop_reason,
                &assertions_for_junit,
                &result.firmware_hash,
                &result.config,
                result.steps_executed,
                result.cycles,
                result.instructions,
            ) {
                error!("Failed to write junit.xml: {}", e);
            }
        }
    }

    if let Some(junit_path) = &args.junit {
        if let Some(parent) = junit_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = write_junit_xml(
            junit_path,
            status,
            duration,
            &result.stop_reason,
            &assertions_for_junit,
            &result.firmware_hash,
            &result.config,
            result.steps_executed,
            result.cycles,
            result.instructions,
        ) {
            error!("Failed to write JUnit report {:?}: {}", junit_path, e);
        }
    }
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

fn resolve_script_path(script_path: &PathBuf, value: &str) -> PathBuf {
    let p = PathBuf::from(value);
    if p.is_absolute() {
        return p;
    }
    script_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(p)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn write_junit_xml(
    path: &Path,
    status: &str,
    duration: std::time::Duration,
    stop_reason: &StopReason,
    assertions: &[AssertionResult],
    firmware_hash: &str,
    config: &TestConfig,
    steps_executed: u64,
    cycles: u64,
    instructions: u64,
) -> std::io::Result<()> {
    let tests = 1;
    let failures = if status == "fail" { 1 } else { 0 };
    let errors = if status == "error" { 1 } else { 0 };

    let mut details = String::new();
    details.push_str(&format!("stop_reason={:?}\n", stop_reason));
    details.push_str(&format!("steps_executed={}\n", steps_executed));
    details.push_str(&format!("cycles={}\n", cycles));
    details.push_str(&format!("instructions={}\n", instructions));
    details.push_str(&format!("firmware_hash={}\n", firmware_hash));
    details.push_str(&format!("firmware={}\n", config.firmware.display()));
    if let Some(sys) = &config.system {
        details.push_str(&format!("system={}\n", sys.display()));
    }
    details.push_str(&format!("script={}\n", config.script.display()));
    if !assertions.is_empty() {
        details.push_str("assertions:\n");
        for a in assertions {
            details.push_str(&format!("  - {:?}: {}\n", a.assertion, a.passed));
        }
    }

    let time_secs = duration.as_secs_f64();

    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(&format!(
        r#"<testsuite name="labwired" tests="{}" failures="{}" errors="{}" time="{:.6}">"#,
        tests, failures, errors, time_secs
    ));
    xml.push('\n');
    xml.push_str("  <properties>\n");
    xml.push_str(&format!(
        "    <property name=\"stop_reason\" value=\"{}\"/>\n",
        xml_escape(&format!("{:?}", stop_reason))
    ));
    xml.push_str(&format!(
        "    <property name=\"firmware_hash\" value=\"{}\"/>\n",
        xml_escape(firmware_hash)
    ));
    xml.push_str("  </properties>\n");
    xml.push_str(&format!(
        "  <testcase classname=\"labwired\" name=\"labwired test\" time=\"{:.6}\">\n",
        time_secs
    ));

    if status == "fail" {
        xml.push_str(&format!(
            "    <failure message=\"assertion failure\">{}</failure>\n",
            xml_escape(&details)
        ));
    } else if status == "error" {
        xml.push_str(&format!(
            "    <error message=\"runtime error\">{}</error>\n",
            xml_escape(&details)
        ));
    }

    xml.push_str("  </testcase>\n");
    xml.push_str("</testsuite>\n");

    std::fs::write(path, xml)
}

// Minimal regex matcher supporting: '^' anchor, '$' anchor, '.' and '*' (Kleene star).
// This is intentionally small to avoid introducing new deps; it does not implement full PCRE/Rust regex.
fn simple_regex_is_match(pattern: &str, text: &str) -> bool {
    fn char_eq(pat: char, ch: char) -> bool {
        pat == '.' || pat == ch
    }

    fn match_here(pat: &[char], text: &[char]) -> bool {
        if pat.is_empty() {
            return true;
        }
        if pat.len() >= 2 && pat[1] == '*' {
            return match_star(pat[0], &pat[2..], text);
        }
        if pat[0] == '$' && pat.len() == 1 {
            return text.is_empty();
        }
        if !text.is_empty() && char_eq(pat[0], text[0]) {
            return match_here(&pat[1..], &text[1..]);
        }
        false
    }

    fn match_star(ch: char, pat: &[char], text: &[char]) -> bool {
        let mut i = 0;
        loop {
            if match_here(pat, &text[i..]) {
                return true;
            }
            if i >= text.len() {
                return false;
            }
            if !char_eq(ch, text[i]) {
                return false;
            }
            i += 1;
        }
    }

    let pat_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    if pat_chars.first().copied() == Some('^') {
        return match_here(&pat_chars[1..], &text_chars);
    }

    for start in 0..=text_chars.len() {
        if match_here(&pat_chars, &text_chars[start..]) {
            return true;
        }
    }
    false
}
