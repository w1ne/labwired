# LabWired Architecture

## High-Level Overview

The system is designed as a set of decoupled Rust crates to ensure portability and separation of concerns.

```mermaid
graph TD
    CLI[sim-cli] --> Loader[sim-loader]
    CLI --> Core[sim-core]
    Loader --> Core
    
    subgraph Core [sim-core]
        CPU[Cortex-M CPU]
        Bus[System Bus]
        Dec[Decoder]
        Mem[Linear Memory]
        
        CPU --> Dec
        CPU --> Bus
        Bus --> Mem
    end
```

## Component Definitions

### 1. `sim-core`
The execution engine. Designed to be `no_std` compatible.

#### **Memory Model**
The system uses a `SystemBus` that routes memory accesses based on the address map:
- **Flash Memory**: `0x0000_0000` (Read-Only via Bus, writable by Loader) - Default 1MB.
- **RAM**: `0x2000_0000` (Read/Write) - Default 128KB.

The underlying storage is `LinearMemory`, a flat `Vec<u8>` optimized for direct access.

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
**Supported Instructions (v0.1.0)**:
- `NOP`: No Operation (`0xBF00`)
- `MOV Rd, #imm8`: Move 8-bit immediate to register (`0x2xxx`)
- `B <offset>`: Unconditional Branch (`0xE...`)

### 2. `sim-loader`
Handles binary parsing.
- Uses `goblin` to parse ELF files.
- Extracts `PT_LOAD` segments.
- Produces a `ProgramImage` containing segments and the Entry Point.

### 3. `sim-cli`
The host runner and entry point.
- **Initialization**: Sets up `tracing` and signal handling.
- **Loading**: Loads ELF into `Machine` memory (mapping segments to Flash/RAM).
- **Simulation**: Runs the `Machine::step()` loop.

