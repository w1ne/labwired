# LabWired Architecture

## High-Level Overview

The system is designed as a set of decoupled Rust crates to ensure portability and separation of concerns.

```mermaid
graph TD
    CLI[labwired-cli] --> Config[labwired-config]
    CLI --> Loader[labwired-loader]
    CLI --> Core[labwired-core]
    Config --> Core
    Loader --> Core
    
    subgraph Core [labwired-core]
        CPU[Cortex-M CPU]
        Bus[System Bus]
        Dec[Decoder]
        Mem[Linear Memory]
        Periphs[Dynamic Peripherals]
        
        CPU --> Dec
        CPU --> Bus
        Bus --> Mem
        Bus --> Periphs
    end
```

## Component Definitions

### 1. `sim-core`
The execution engine. Designed to be `no_std` compatible and **architecture-agnostic**.

#### **Pluggable Core Pattern**
The `Machine` struct is generic over the `Cpu` trait (`Machine<C: Cpu>`). This allows swapping the execution core (e.g., specific Cortex-M variants, RISC-V, etc.) without changing the bus or memory infrastructure.
The `Cpu` trait defines the minimal interface:
```rust
trait Cpu {
    fn reset(&mut self);
    fn step(&mut self, bus: &mut dyn Bus) -> SimResult<()>;
}
```

#### **Memory Model**

#### **Dynamic Bus & Peripherals**
The system uses a `SystemBus` that routes memory accesses dynamically based on a project manifest.
- **Flash Memory**: Base address varies by chip. Loads ELF segments.
- **RAM**: Base address varies by chip. Supports read/write.
- **Peripherals**: Memory-mapped devices (UART, SysTick, Stubs) mapped to arbitrary address ranges.

Peripherals are integrated via the `Peripheral` trait:
```rust
pub trait Peripheral: std::fmt::Debug + Send {
    fn read(&self, offset: u64) -> SimResult<u8>;
    fn write(&mut self, offset: u64, value: u8) -> SimResult<()>;
    fn tick(&mut self) -> PeripheralTickResult; // Returns IRQ status, cycles, and DMA requests
}
```

#### **CPU (Cortex-M Stub)**
Represents the processor state.
- **Registers**:
    - `R0-R12`: General Purpose
    - `SP (R13)`: Stack Pointer
    - `LR (R14)`: Link Register
    - `PC (R15)`: Program Counter
    - `xPSR`: Program Status Register
- **Execution Cycle**:
    1.  **Fetch**: Read 16-bit Opcode from `PC` via `Bus`.
    2.  **Decode**: Translate Opcode into `Instruction` enum via `Decoder`.
    3.  **Execute**: Update PC/Registers based on `Instruction`.

#### **Decoder (Thumb-2)**
A stateless module confirming to ARMv7-M Thumb-2 encoding.
**Supported Instructions**:
- **Control Flow**: `B <offset>`, `Bcc <cond, offset>`, `BL` (32-bit), `BX`, `CBZ`, `CBNZ`.
- **Arithmetic**: `ADD`, `SUB`, `CMP`, `MOV`, `MVN`, `MOVW` (32-bit), `MOVT` (32-bit), `MUL`.
    - **Wide (32-bit) Variants**: `ADD.W`, `SUB.W`, `ADC.W`, `SBC.W`, `MOV.W`, `MVN.W`, `BIC.W`, `ORN.W`.
    - **Shifted Register Variants**: Support for 32-bit forms with arbitrary barrel shifter offsets.
    - **Division**: `SDIV`, `UDIV` (32-bit encoding).
    - Includes **High Register** support for `MOV`, `CMP`, and `ADD`.
    - Dedicated `ADD SP, #imm` and `SUB SP, #imm` forms.
- **Logic**: `AND`, `ORR`, `EOR`.
- **Shifts**: `LSL`, `LSR`, `ASR`, `ROR` (Immediate and Register-modified).
- **Bit Field & Misc**: `BFI`, `BFC`, `SBFX`, `UBFX`, `CLZ`, `RBIT`, `REV`, `REV16`, `UXTB`.
- **Memory**:
    - `LDR`/`STR` (Immediate Offset / Word)
    - `LDRB`/`STRB` (Immediate Offset / Byte)
    - `LDRH`/`STRH` (Immediate Offset / Halfword)
    - `LDR` (Literal / PC-Relative)
    - `LDR`/`STR` (SP-Relative)
    - `LDRD`/`STRD` (Double Word - minimal support)
    - `PUSH`/`POP` (Stack Operations)
- **Interrupt Control**: `CPSIE`, `CPSID` (affecting `primask`).
- **Other**: `NOP`, `IT` block (treated as NOP hints for robustness).

#### **Core Peripherals (STM32F1 Compatible)**
The system includes a suite of memory-mapped peripherals to support real-world HAL libraries:
- **GPIO**: Mode configuration (CRL/CRH), Pin state tracking (IDR/ODR), and atomic bit manipulation (BSRR/BRR).
- **RCC**: Reset and Clock Control (minimal) for peripheral enablement.
- **Timers**: TIM2/TIM3 general-purpose timers with prescaling and update interrupts.
- **I2C**: I2C Master mode with status flag sequence support (Start, Address, Transmit).
- **SPI**: SPI Master mode with basic full-duplex transfer mocking.
- **SysTick**: Standard Cortex-M system timer.
- **NVIC**: Nested Vectored Interrupt Controller with prioritization and masking.
- **SCB**: System Control Block with VTOR support.
- **DMA1**: 7-channel Direct Memory Access controller with bus mastering support.
- **EXTI**: External Interrupt/Event Controller for handling external signals.
- **AFIO**: Alternate Function I/O for dynamic pin-to-interrupt mapping.

### DMA Mastering (Two-Phase Execution)
To maintain modularity and comply with Rust's ownership rules, LabWired uses a two-phase execution model for DMA:
1.  **Phase 1 (Request)**: During `tick()`, a peripheral returns a list of `DmaRequest`s.
2.  **Phase 2 (Execute)**: The `SystemBus` iterates over these requests and performs the corresponding memory operations.

#### **32-bit Reassembly**
The CPU supports robust reassembly of 32-bit Thumb-2 instructions (`BL`, `MOVW`, `MOVT`, `MOV.W`, `MVN.W`, `SDIV`, `UDIV`) by fetching the suffix half-word during the execution of a `Prefix32` opcode.

### 2. `labwired-config`
Handles hardware declaration and validation.
- **Schemas**: Defines `ChipDescriptor` and `SystemManifest` (YAML).
- **Size Parsing**: Converts human-readable strings like "128KB" to raw byte sizes.
- **Dependency**: Used by `CLI` to initialize the `Machine` and by `Core` to map peripherals.

### 3. `labwired-loader`
Handles binary parsing.
- Uses `goblin` to parse ELF files.
- Extracts `PT_LOAD` segments.
- Produces a `ProgramImage` containing segments and the Entry Point.

### 4. `labwired-cli`
The host runner and entry point.
- **Initialization**: Parses `--firmware` and optional `--system` manifest.
- **Configuration**: Resolves Chip Descriptors and wiring via `labwired-config`.
- **Loading**: Loads ELF segments into the dynamically configured `SystemBus`.
- **Simulation**: Runs the `Machine::step()` loop.

