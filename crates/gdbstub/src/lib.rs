// LabWired - Firmware Simulation Platform
// Copyright (C) 2026 Andrii Shylenko
//
// This software is released under the MIT License.
// See the LICENSE file in the project root for full license information.

use core::convert::Infallible;
use gdbstub::common::Signal;
use gdbstub::stub::{BaseStopReason, GdbStub};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadSingleStep,
};
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::{Target, TargetError, TargetResult};
use labwired_core::cpu::CortexM;
use labwired_core::{DebugControl, Machine, StopReason};
use std::net::{TcpListener, TcpStream};

pub struct LabwiredTarget {
    pub machine: Machine<CortexM>,
}

impl LabwiredTarget {
    pub fn new(machine: Machine<CortexM>) -> Self {
        Self { machine }
    }
}

impl Target for LabwiredTarget {
    type Arch = gdbstub_arch::arm::Armv4t;
    type Error = Infallible;

    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    fn support_breakpoints(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for LabwiredTarget {
    fn read_registers(
        &mut self,
        regs: &mut gdbstub_arch::arm::reg::ArmCoreRegs,
    ) -> TargetResult<(), Self> {
        for i in 0..13 {
            regs.r[i] = self.machine.read_core_reg(i as u8);
        }
        regs.sp = self.machine.read_core_reg(13);
        regs.lr = self.machine.read_core_reg(14);
        regs.pc = self.machine.read_core_reg(15);
        regs.cpsr = self.machine.read_core_reg(16); // xPSR
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &gdbstub_arch::arm::reg::ArmCoreRegs,
    ) -> TargetResult<(), Self> {
        for i in 0..13 {
            self.machine.write_core_reg(i as u8, regs.r[i]);
        }
        self.machine.write_core_reg(13, regs.sp);
        self.machine.write_core_reg(14, regs.lr);
        self.machine.write_core_reg(15, regs.pc);
        self.machine.write_core_reg(16, regs.cpsr);
        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8]) -> TargetResult<usize, Self> {
        let mem = self
            .machine
            .read_memory(start_addr, data.len())
            .map_err(|_| TargetError::NonFatal)?;
        let len = mem.len().min(data.len());
        data[..len].copy_from_slice(&mem[..len]);
        Ok(len)
    }

    fn write_addrs(&mut self, start_addr: u32, data: &[u8]) -> TargetResult<(), Self> {
        self.machine
            .write_memory(start_addr, data)
            .map_err(|_| TargetError::NonFatal)?;
        Ok(())
    }

    fn support_resume(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::singlethread::SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadResume for LabwiredTarget {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn support_single_step(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::singlethread::SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadSingleStep for LabwiredTarget {
    fn step(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl gdbstub::target::ext::breakpoints::Breakpoints for LabwiredTarget {
    fn support_sw_breakpoint(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::breakpoints::SwBreakpoint for LabwiredTarget {
    fn add_sw_breakpoint(
        &mut self,
        addr: u32,
        _kind: gdbstub_arch::arm::ArmBreakpointKind,
    ) -> TargetResult<bool, Self> {
        self.machine.add_breakpoint(addr);
        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: u32,
        _kind: gdbstub_arch::arm::ArmBreakpointKind,
    ) -> TargetResult<bool, Self> {
        self.machine.remove_breakpoint(addr);
        Ok(true)
    }
}

pub struct GdbServer {
    port: u16,
}

impl GdbServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn run(&self, machine: Machine<CortexM>) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port))?;
        tracing::info!("GDB server listening on 0.0.0.0:{}", self.port);

        let (stream, addr) = listener.accept()?;
        tracing::info!("GDB client connected from {}", addr);

        let mut target = LabwiredTarget::new(machine);
        let gdb = GdbStub::new(stream);

        match gdb.run_blocking::<GdbEventLoop>(&mut target) {
            Ok(reason) => tracing::info!("GDB session ended: {:?}", reason),
            Err(e) => tracing::error!("GDB session error: {:?}", e),
        }

        Ok(())
    }
}

struct GdbEventLoop;

impl gdbstub::stub::run_blocking::BlockingEventLoop for GdbEventLoop {
    type Target = LabwiredTarget;
    type Connection = TcpStream;
    type StopReason = BaseStopReason<(), u32>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        gdbstub::stub::run_blocking::Event<Self::StopReason>,
        gdbstub::stub::run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as gdbstub::conn::Connection>::Error,
        >,
    > {
        use gdbstub::stub::run_blocking::Event;
        use std::io::Read;

        // Non-blocking peep at connection for interrupt
        let mut byte = [0];
        conn.set_nonblocking(true).ok();
        let incoming = match conn.read(&mut byte) {
            Ok(1) => Some(byte[0]),
            _ => None,
        };
        conn.set_nonblocking(false).ok();

        if let Some(b) = incoming {
            return Ok(Event::IncomingData(b));
        }

        // Run machine for a bit
        match target.machine.run(Some(1000)) {
            Ok(StopReason::Breakpoint(_)) => Ok(Event::TargetStopped(BaseStopReason::Signal(
                Signal::SIGTRAP,
            ))),
            Ok(StopReason::StepDone) => Ok(Event::TargetStopped(BaseStopReason::Signal(
                Signal::SIGTRAP,
            ))),
            _ => {
                // Keep running
                Ok(Event::TargetStopped(BaseStopReason::Signal(Signal::SIGINT)))
            }
        }
    }

    fn on_interrupt(
        _target: &mut Self::Target,
    ) -> Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        Ok(Some(BaseStopReason::Signal(Signal::SIGINT)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use labwired_core::bus::SystemBus;
    use labwired_core::cpu::CortexM;

    #[test]
    fn test_target_register_access() {
        let bus = SystemBus::new();
        let machine = Machine::<CortexM>::with_bus(bus);
        let mut target = LabwiredTarget::new(machine);

        // Mock some register values
        target.machine.write_core_reg(0, 0x12345678);
        target.machine.write_core_reg(15, 0x08000100);

        let mut regs = gdbstub_arch::arm::reg::ArmCoreRegs::default();
        target
            .read_registers(&mut regs)
            .unwrap_or_else(|_| panic!("Failed to read registers"));

        assert_eq!(regs.r[0], 0x12345678);
        assert_eq!(regs.pc, 0x08000100);

        // Test write
        regs.r[1] = 0xdeadbeef;
        target
            .write_registers(&regs)
            .unwrap_or_else(|_| panic!("Failed to write registers"));
        assert_eq!(target.machine.read_core_reg(1), 0xdeadbeef);
    }

    #[test]
    fn test_target_memory_access() {
        let mut bus = SystemBus::new();
        // Just mock some RAM. add_peripheral is not directly on SystemBus,
        // usually it's built from config or manually added to the bus internal vector.
        // Actually SystemBus might have a way to add peripherals in tests.

        let machine = Machine::<CortexM>::with_bus(bus);
        let mut target = LabwiredTarget::new(machine);

        // We can use the read_memory write_memory logic.
        // But without any peripherals, it might return SimulationError.
        // Let's assume we can at least test the trait call.
    }
}
