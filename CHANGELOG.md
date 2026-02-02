# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
