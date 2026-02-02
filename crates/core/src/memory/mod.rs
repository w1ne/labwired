use serde::{Serialize, Deserialize};
use crate::SimResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub start_addr: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramImage {
    pub entry_point: u64,
    pub segments: Vec<Segment>,
}

impl ProgramImage {
    pub fn new(entry_point: u64) -> Self {
        Self {
            entry_point,
            segments: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, start_addr: u64, data: Vec<u8>) {
        self.segments.push(Segment { start_addr, data });
    }
}

/// A simple flat memory storage
pub struct LinearMemory {
    pub data: Vec<u8>,
    pub base_addr: u64,
}

impl LinearMemory {
    pub fn new(size: usize, base_addr: u64) -> Self {
        Self {
            data: vec![0; size],
            base_addr,
        }
    }

    pub fn read_u8(&self, addr: u64) -> Option<u8> {
        if addr >= self.base_addr && addr < self.base_addr + self.data.len() as u64 {
            Some(self.data[(addr - self.base_addr) as usize])
        } else {
            None
        }
    }

    pub fn write_u8(&mut self, addr: u64, value: u8) -> bool {
        if addr >= self.base_addr && addr < self.base_addr + self.data.len() as u64 {
            self.data[(addr - self.base_addr) as usize] = value;
            true
        } else {
            false
        }
    }
    
    pub fn load_from_segment(&mut self, segment: &Segment) -> bool {
        // Simple overlap check
        let end_addr = segment.start_addr + segment.data.len() as u64;
        let mem_end = self.base_addr + self.data.len() as u64;
        
        if segment.start_addr >= self.base_addr && end_addr <= mem_end {
             let offset = (segment.start_addr - self.base_addr) as usize;
             self.data[offset..offset + segment.data.len()].copy_from_slice(&segment.data);
             return true;
        }
        false
    }
}
