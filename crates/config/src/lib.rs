use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryRange {
    pub base: u64,
    pub size: String, // e.g. "128KB"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeripheralConfig {
    pub id: String,
    pub r#type: String, // "uart", "timer", "gpio", etc.
    pub base_address: u64,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub irq: Option<u32>,
    #[serde(default)]
    pub config: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChipDescriptor {
    pub name: String,
    pub arch: String, // e.g. "cortex-m3"
    pub flash: MemoryRange,
    pub ram: MemoryRange,
    pub peripherals: Vec<PeripheralConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalDevice {
    pub id: String,
    pub r#type: String,
    pub connection: String, // e.g. "uart1", "i2c1"
    #[serde(default)]
    pub config: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemManifest {
    pub name: String,
    pub chip: String, // Reference to chip name or file path
    #[serde(default)]
    pub memory_overrides: HashMap<String, String>,
    #[serde(default)]
    pub external_devices: Vec<ExternalDevice>,
}

impl ChipDescriptor {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = std::fs::File::open(path)?;
        serde_yaml::from_reader(f).context("Failed to parse Chip Descriptor")
    }
}

impl SystemManifest {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = std::fs::File::open(path)?;
        serde_yaml::from_reader(f).context("Failed to parse System Manifest")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestInputs {
    pub firmware: String,
    pub system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestLimits {
    pub max_steps: u64,
    #[serde(default)]
    pub wall_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    MaxSteps,
    WallTime,
    MemoryViolation,
    DecodeError,
    Halt,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct UartContainsAssertion {
    pub uart_contains: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct UartRegexAssertion {
    pub uart_regex: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StopReasonAssertion {
    pub expected_stop_reason: StopReason,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TestAssertion {
    UartContains(UartContainsAssertion),
    UartRegex(UartRegexAssertion),
    ExpectedStopReason(StopReasonAssertion),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestScript {
    pub schema_version: String,
    pub inputs: TestInputs,
    pub limits: TestLimits,
    #[serde(default)]
    pub assertions: Vec<TestAssertion>,
}

impl TestScript {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = std::fs::File::open(&path)
            .with_context(|| format!("Failed to open test script at {:?}", path.as_ref()))?;
        let script: Self = serde_yaml::from_reader(f)
            .context("Failed to parse Test Script YAML")?;
        script.validate()?;
        Ok(script)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != "1.0" {
            anyhow::bail!(
                "Unsupported schema_version '{}'. Supported versions: '1.0'",
                self.schema_version
            );
        }

        if self.inputs.firmware.trim().is_empty() {
            anyhow::bail!("Input 'firmware' path cannot be empty");
        }

        if self.limits.max_steps == 0 {
            anyhow::bail!("Limit 'max_steps' must be greater than zero");
        }

        Ok(())
    }
}

pub fn parse_size(size_str: &str) -> Result<u64> {
    use human_size::{Byte, Size, SpecificSize};
    let s: Size = size_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid size format: {}", e))?;
    let bytes: SpecificSize<Byte> = s.into();
    Ok(bytes.value() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_script() {
        let yaml = r#"
schema_version: "1.0"
inputs:
  firmware: "path/to/fw.elf"
  system: "path/to/sys.yaml"
limits:
  max_steps: 1000
  wall_time_ms: 5000
assertions:
  - uart_contains: "Hello"
  - expected_stop_reason: halt
"#;
        let script: TestScript = serde_yaml::from_str(yaml).unwrap();
        assert!(script.validate().is_ok());
        assert_eq!(script.inputs.firmware, "path/to/fw.elf");
        assert_eq!(script.limits.max_steps, 1000);
        assert_eq!(script.assertions.len(), 2);
    }

    #[test]
    fn test_invalid_version() {
        let yaml = r#"
schema_version: "2.0"
inputs:
  firmware: "fw.elf"
limits:
  max_steps: 100
"#;
        let script: TestScript = serde_yaml::from_str(yaml).unwrap();
        let err = script.validate().unwrap_err();
        assert!(err.to_string().contains("Unsupported schema_version"));
    }

    #[test]
    fn test_invalid_max_steps() {
        let yaml = r#"
schema_version: "1.0"
inputs:
  firmware: "fw.elf"
limits:
  max_steps: 0
"#;
        let script: TestScript = serde_yaml::from_str(yaml).unwrap();
        let err = script.validate().unwrap_err();
        assert!(err.to_string().contains("max_steps"));
    }

    #[test]
    fn test_empty_firmware() {
        let yaml = r#"
schema_version: "1.0"
inputs:
  firmware: ""
limits:
  max_steps: 100
"#;
        let script: TestScript = serde_yaml::from_str(yaml).unwrap();
        let err = script.validate().unwrap_err();
        assert!(err.to_string().contains("firmware"));
    }
}
