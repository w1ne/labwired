use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_cli_interactive_writes_snapshot() {
    let firmware = std::fs::canonicalize("../../tests/fixtures/uart-ok-thumbv7m.elf").unwrap();

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let snapshot_path =
        std::env::temp_dir().join(format!("labwired-interactive-snapshot-{}.json", nonce));
    let _ = std::fs::remove_file(&snapshot_path);

    let output = Command::new(env!("CARGO_BIN_EXE_labwired"))
        .args([
            "--firmware",
            firmware.to_str().unwrap(),
            "--max-steps",
            "1",
            "--snapshot",
            snapshot_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute labwired");

    assert!(output.status.success());
    assert!(snapshot_path.exists());

    let snapshot_content = std::fs::read_to_string(&snapshot_path).unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(&snapshot_content).unwrap();
    assert_eq!(snapshot["type"], "interactive_cortex_m");

    let regs = snapshot["cpu"]["registers"].as_array().unwrap();
    assert_eq!(regs.len(), 16);

    let _ = std::fs::remove_file(&snapshot_path);
}
