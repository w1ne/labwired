use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct MachineSnapshot {
    pub cpu: CpuSnapshot,
    pub peripherals: HashMap<String, serde_json::Value>,
    // Future: metrics
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuSnapshot {
    pub registers: [u32; 16],
    pub xpsr: u32,
    pub primask: bool,
    pub pending_exceptions: u32,
    pub vtor: u32,
}
