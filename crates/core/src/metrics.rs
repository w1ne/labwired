use crate::SimulationObserver;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug)]
pub struct PerformanceMetrics {
    instruction_count: AtomicU64,
    cycle_count: AtomicU64,
    start_time: Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            instruction_count: AtomicU64::new(0),
            cycle_count: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn reset(&self) {
        self.instruction_count.store(0, Ordering::SeqCst);
        self.cycle_count.store(0, Ordering::SeqCst);
    }

    pub fn get_instructions(&self) -> u64 {
        self.instruction_count.load(Ordering::SeqCst)
    }

    pub fn get_cycles(&self) -> u64 {
        self.cycle_count.load(Ordering::SeqCst)
    }

    pub fn get_ips(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.get_instructions() as f64 / elapsed
        } else {
            0.0
        }
    }
}

impl SimulationObserver for PerformanceMetrics {
    fn on_simulation_start(&self) {
        // Reset counters on each start if needed, or just keep them cumulative
    }

    fn on_step_start(&self, _pc: u32, _opcode: u16) {
        self.instruction_count.fetch_add(1, Ordering::SeqCst);
    }

    fn on_step_end(&self, cycles: u32) {
        self.cycle_count.fetch_add(cycles as u64, Ordering::SeqCst);
    }
}
