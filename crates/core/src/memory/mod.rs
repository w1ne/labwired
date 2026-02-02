use serde::{Serialize, Deserialize};

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
