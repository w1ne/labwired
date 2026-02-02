use std::process::Command;
use std::path::Path;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("LabWired Simulator"));
}

#[test]
fn test_cli_load_missing_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .arg("-f")
        .arg("non_existent_file.elf")
        .output()
        .expect("Failed to execute command");

    // It should fail because file is missing
    assert!(!output.status.success());
}
