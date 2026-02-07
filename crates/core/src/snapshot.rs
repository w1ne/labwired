// LabWired - Firmware Simulation Platform
// Copyright (C) 2026 Andrii Shylenko
//
// This software is released under the MIT License.
// See the LICENSE file in the project root for full license information.

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
    pub registers: Vec<u32>,
    pub xpsr: u32,
    pub primask: bool,
    pub pending_exceptions: u32,
    pub vtor: u32,
}
