use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_file(prefix: &str, contents: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push("labwired-tests");
    let _ = std::fs::create_dir_all(&dir);

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = dir.join(format!("{}-{}.yaml", prefix, nonce));
    std::fs::write(&path, contents).expect("Failed to write temp file");
    path
}

#[test]
fn test_cli_test_mode_outputs() {
    let mut dir = std::env::temp_dir();
    dir.push("labwired-tests-outputs");
    let _ = std::fs::create_dir_all(&dir);

    // Copy dummy.elf to the temp dir to test relative path resolution
    let fw_path = dir.join("dummy.elf");
    std::fs::copy("../../tests/dummy.elf", &fw_path).expect("Failed to copy dummy.elf");

    let script_path = dir.join("script.yaml");
    let script_content = r#"
schema_version: "1.0"
inputs:
  firmware: "dummy.elf"
limits:
  max_steps: 10
assertions:
  - uart_regex: ".*"
  - expected_stop_reason: max_steps
"#;
    std::fs::write(&script_path, script_content).expect("Failed to write script");

    let output_dir = dir.join("artifacts");

    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .args([
            "test",
            "--script",
            script_path.to_str().unwrap(),
            "--no-uart-stdout",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let result_path = output_dir.join("result.json");
    assert!(result_path.exists());

    let junit_path = output_dir.join("junit.xml");
    assert!(junit_path.exists());
    let junit = std::fs::read_to_string(&junit_path).unwrap();
    assert!(junit.contains("<testsuite"));
    assert!(junit.contains("<testcase"));

    let result_content = std::fs::read_to_string(&result_path).unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_content).unwrap();

    assert_eq!(result["status"], "pass");
    assert_eq!(result["stop_reason"], "max_steps");
    assert!(result["firmware_hash"].as_str().is_some());
    assert!(result["config"]["firmware"]
        .as_str()
        .unwrap()
        .contains("dummy.elf"));

    // Clean up
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_cli_test_mode_junit_flag_writes_file() {
    let fw_abs = std::fs::canonicalize("../../tests/dummy.elf").unwrap();
    let script = write_temp_file(
        "script-junit-path",
        &format!(
            r#"
schema_version: "1.0"
inputs:
  firmware: "{}"
limits:
  max_steps: 1
assertions:
  - expected_stop_reason: max_steps
"#,
            fw_abs.to_str().unwrap()
        ),
    );

    let junit_path = std::env::temp_dir().join("labwired-junit-flag.xml");
    let _ = std::fs::remove_file(&junit_path);

    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .args([
            "test",
            "--script",
            script.to_str().unwrap(),
            "--no-uart-stdout",
            "--junit",
            junit_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(junit_path.exists());

    let junit = std::fs::read_to_string(&junit_path).unwrap();
    assert!(junit.contains("<testsuite"));
    assert!(junit.contains("labwired test"));
}

#[test]
fn test_cli_test_mode_wall_time() {
    let fw_abs = std::fs::canonicalize("../../tests/dummy.elf").unwrap();
    let script = write_temp_file(
        "script-walltime",
        &format!(
            r#"
schema_version: "1.0"
inputs:
  firmware: "{}"
limits:
  max_steps: 10000000
  wall_time_ms: 0
assertions:
  - expected_stop_reason: wall_time
"#,
            fw_abs.to_str().unwrap()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .args([
            "test",
            "--script",
            script.to_str().unwrap(),
            "--no-uart-stdout",
        ])
        .output()
        .expect("Failed to execute command");

    // Should pass because we expect wall_time stop reason
    assert!(output.status.success());
}

#[test]
fn test_cli_test_mode_memory_violation() {
    let fw_abs = std::fs::canonicalize("../../tests/dummy.elf").unwrap();
    let script = write_temp_file(
        "script-memviol",
        &format!(
            r#"
schema_version: "1.0"
inputs:
  firmware: "{}"
limits:
  max_steps: 1000
assertions:
  - expected_stop_reason: memory_violation
"#,
            fw_abs.to_str().unwrap()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .args([
            "test",
            "--script",
            script.to_str().unwrap(),
            "--no-uart-stdout",
        ])
        .output()
        .expect("Failed to execute command");

    // Should pass because we expect memory_violation stop reason
    assert!(output.status.success());
}

#[test]
fn test_cli_test_mode_max_steps_guard() {
    let fw_abs = std::fs::canonicalize("../../tests/dummy.elf").unwrap();
    let script = write_temp_file(
        "script-huge",
        &format!(
            r#"
schema_version: "1.0"
inputs:
  firmware: "{}"
limits:
  max_steps: 60000000
"#,
            fw_abs.to_str().unwrap()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .args(["test", "--script", script.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should fail due to MAX_ALLOWED_STEPS guard
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2)); // EXIT_CONFIG_ERROR
}

#[test]
fn test_cli_test_mode_regex_fail() {
    let fw_abs = std::fs::canonicalize("../../tests/dummy.elf").unwrap();
    let script = write_temp_file(
        "script-regex-fail",
        &format!(
            r#"
schema_version: "1.0"
inputs:
  firmware: "{}"
limits:
  max_steps: 10
assertions:
  - uart_regex: "^ThisTextWillNeverBeFound$"
"#,
            fw_abs.to_str().unwrap()
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .args(["test", "--script", script.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1)); // EXIT_ASSERT_FAIL
}
