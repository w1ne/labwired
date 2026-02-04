# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2026-02-04

### Added
- **Testing Infrastructure**: 
    - **Test Script Schema (YAML)**: Versioned schema for defining firmware tests with inputs (ELF/System), limits (steps/time), and assertions (UART contents, stop reasons).
    - **CI Regression Gates**: Enforced workspace-wide testing and linting in GitHub Actions.
    - **Pre-Release Verification**: Automated regression suite execution on release tags and PRs.
- **Documentation**: 
    - Updated `README.md` to reflect real-world division firmware behavior and IPS reporting.
    - Updated `plan.md` Iteration 10 with implementation details for modular observability.

## [0.8.0] - 2026-02-03

### Added
- **Observability**: Modular metrics and simulation instrumentation:
    - **SimulationObserver Trait**: Pluggable architecture for observing simulation events (reset, step, start/stop).
    - **PerformanceMetrics**: Thread-safe instruction and cycle tracking using atomic counters.
    - **Real-Time IPS**: CLI reports simulation speed (Instructions Per Second) and progress updates.
- **Modularity**: Decoupled introspection tools from the core execution engine, enabling zero-overhead simulation when observers are detached.
- **Tests**: 
    - **test_metrics_collection**: Verified cycle accuracy for 16-bit and 32-bit (BL) instructions.

## [0.7.0] - 2026-02-03

### Added
- **ISA**: Advanced Thumb-2 instruction set extensions for HAL compatibility:
    - **MOV.W / MVN.W (T2/T3)**: 32-bit move/move-not with ARM-modified immediate expansion.
    - **SDIV / UDIV (T1)**: Signed and Unsigned 32-bit division instructions.
    - **thumb_expand_imm()**: Recursive immediate constant expansion for 32-bit instructions.
- **Core Peripherals**: STM32F1-compatible memory-mapped peripheral ecosystem:
    - **GPIO**: Mode config (CRL/CRH), Pin state tracking (IDR/ODR), and atomic bit manipulation (BSRR/BRR).
    - **RCC**: Reset & Clock Control enabling peripheral lifecycle management.
    - **Timers (TIM2/TIM3)**: 16-bit timers with prescaling and update interrupts.
    - **I2C**: Master mode support with status flags (SB, ADDR, TXE, etc.).
    - **SPI**: Master mode transfer simulation and status management.
- **CLI**: Advanced simulation and debugging features:
    - **Execution Tracing**: `--trace` flag for instruction-level logging with PC and opcode.
    - **Simulation Control**: `--max-steps` option to prevent infinite loops in firmware.
- **Diagnostics**: Detailed error hinting for unknown instructions (Thumb-2 vs Coprocessor vs SIMD).
- **Tests**: Comprehensive validation suite:
    - `test_mov_w_instruction` & `test_mvn_w_instruction`.
    - `test_division_instructions` for SDIV/UDIV.
    - `test_gpio_basic` for peripheral register and bit manipulation verification.
    - Total unit tests: **37**.

### Changed
- Unified 32-bit instruction reassembly logic for broader ISA support.
- Refactored `SystemBus` to pre-register core peripherals (GPIO, RCC, Timers) by default.

## [0.6.0] - 2026-02-03

### Added
- **ISA**: Real-world compatibility instruction set extensions:
    - **Block Memory Operations**: Implemented `LDM` and `STM` for efficient multi-register load/store.
    - **Halfword Access**: Added `LDRH` and `STRH` for 16-bit peripheral register access.
    - **Multiplication**: Implemented `MUL` instruction with N/Z flag updates.
- **System Peripherals**:
    - **NVIC** (Nested Vectored Interrupt Controller) at `0xE000E100`:
        - ISER/ICER registers for interrupt enable/disable
        - ISPR/ICPR registers for interrupt pending management
        - Atomic shared state architecture for thread-safe operation
    - **SCB** (System Control Block) at `0xE000ED00`:
        - VTOR (Vector Table Offset Register) support for runtime relocation
        - Shared atomic state between CPU and memory-mapped peripheral
- **Interrupt Architecture**:
    - Two-phase interrupt delivery (pend → signal) with NVIC filtering
    - External interrupts (IRQ ≥ 16) managed by NVIC ISER/ISPR
    - Core exceptions (< 16) bypass NVIC for architectural compliance
    - VTOR-based exception handler lookup in CPU
- **Bus**: Implemented `read_u16`/`write_u16` for halfword memory access
- **Tests**: Added 3 new system tests (`test_iteration_8_instructions`, `test_nvic_external_interrupt`, `test_vtor_relocation`)

### Fixed
- **Memory Map**: Corrected peripheral size allocations to prevent overlaps (SysTick: 0x10, NVIC: 0x400, SCB: 0x40)
- **CPU**: VTOR now preserved across reset for simulation flexibility

## [0.5.0] - 2026-02-03

### Added
- **ISA**: Advanced instruction support for complex C/C++ firmware initialization:
    - **Stack Manipulation**: Implemented `ADD SP, #imm` and `SUB SP, #imm` (Thumb-2 T1/T2).
    - **High Register Arithmetic**: Extended `ADD` to support high registers (R8-R15), essential for stack frame teardown.
    - **Interrupt Control**: Added `CPSIE` and `CPSID` for global interrupt enable/disable.
- **CPU**: Integrated `primask` register to track and manage global interrupt masking state.
- **Verification**: Expanded unit test suite and verified full `cortex-m-rt` boot flow compatibility.

### Fixed
- **Decoder**: Resolved opcode shadowing for `ADD` (High Register) instructions.
- **Firmware**: Updated UART1 addressing in firmware to align with STM32F103 standard descriptor.

## [0.4.0] - 2026-02-02

### Added
- **System**: Declarative hardware configuration via **System Descriptors**:
    - **Chip Descriptors**: Define SoC architecture (Flash/RAM mapping, Peripheral offsets).
    - **System Manifest**: Describe board-level wiring and external component stubs.
- **Peripherals**: 
    - Full **SysTick** timer implementation (`0xE000_E010`).
    - **StubPeripheral** for functional sensor and device modeling.
- **Core**: 
    - **Vector Table Boot**: Automatic loading of initial SP and PC from address `0x0`.
    - **Exception Lifecycle**: Architectural stacking and unstacking for hardware interrupts.
    - **Dynamic Bus**: Refactored `SystemBus` to support pluggable, manifest-defined components.
- **Crates**: New `labwired-config` crate for YAML-based hardware definitions.

### Changed
- CLI now supports `--system <path>` to load custom hardware configurations.
- Peripheral interaction unified under the `Peripheral` trait.

## [0.1.0] - 2026-02-02

### Added
- **Core**: Initial `Machine`, `Cpu`, `SystemBus` implementation.
- **Loader**: ELF binary parsing support via `goblin`.
- **Decoder**: Basic Thumb-2 decoder supporting `MOV`, `B`, and `NOP`.
- **Memory**: Linear memory model with Flash (0x0) and RAM (0x2...) mapping.
- **CLI**: `labwired-cli` runnable for loading and simulating firmware.
- **Tests**: Dockerized test infrastructure and unit test suite.
- **Docs**: Comprehensive Architecture and Implementation Plan.

### Infrastructure
- CI/CD pipelines via GitHub Actions.
- Dockerfile for portable testing.

## [0.3.0] - 2026-02-02

### Added
- **ISA**: Completing critical instruction set gaps for professional firmware simulation:
    - **32-bit Support**: Implemented 32-bit instruction reassembly logic in CPU fetch loop.
    - **Advanced Data**: Added `MOVW` & `MOVT` for 32-bit immediate loading (enabling peripheral addressing).
    - **Control Flow**: Robust 24-bit Branch with Link (`BL`) reassembly and execution.
    - **Core Support**: Expanded `MOV` & `CMP` to support high registers (R8-R15).
    - **Byte Access**: Implemented `STRB` & `LDRB` for character and buffer handling.
- **Milestone**: Successfully achieved "Hello, LabWired!" simulation output via UART peripheral.

### Fixed
- **ISA**: Corrected `MOV` (High register) decoding logic.
- **Simulation**: Fixed incorrect immediate reassembly order for `MOVW/MOVT` instructions.

## [0.2.0] - 2026-02-02

### Added
- **ISA**: Expanded Instruction Set for robust firmware simulation:
    - Arithmetic: `ADD`, `SUB`, `CMP`, `MOV`, `MVN`.
    - Logic: `AND`, `ORR`, `EOR`.
    - Shifts: `LSL`, `LSR`, `ASR` (immediate).
    - Memory: `LDR` & `STR` (Immediate Offset), `LDR` (Literal), `LDR` & `STR` (SP-relative).
    - Stack & Control: `PUSH`, `POP`, `BL`, `BX`, and Conditional Branches (`Bcc`).
- **Peripherals**: UART stub implementation mapped to `0x4000_C000`.
- **Firmware**: Added `crates/firmware` demo project targeting `thumbv7m-none-eabi`.
- **Core**: Refactored `Machine` to be architecture-agnostic (Pluggable Core).

### Fixed
- **Build**: Resolved ELF load offset issue by correctly configuring workspace-level linker scripts (`link.x`).
- **ISA**: Fixed potential overflow in large immediate offsets for `LDR/STR` instructions.

### Changed
- `labwired-cli` now runs 20,000 steps by default to support firmware boot.
- Updated `docs/architecture.md` and `README.md` with new capabilities.
