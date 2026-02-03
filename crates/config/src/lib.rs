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

pub fn parse_size(size_str: &str) -> Result<u64> {
    use human_size::{Byte, Size, SpecificSize};
    let s: Size = size_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid size format: {}", e))?;
    let bytes: SpecificSize<Byte> = s.into();
    Ok(bytes.value() as u64)
}
