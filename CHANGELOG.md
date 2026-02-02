# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [0.2.0] - 2026-02-02

### Added
- **ISA**: Fully implemented MVP Instruction Set:
    - Arithmetic: `ADD`, `SUB`, `CMP`, `MOV`, `MVN`.
    - Logic: `AND`, `ORR`, `EOR`.
    - Memory: `LDR` & `STR` (Immediate Offset), `LDR` (Literal).
    - Stack & Control: `PUSH`, `POP`, `BL`, `BX`.
- **Peripherals**: UART stub implementation mapped to `0x4000_C000`.
- **Firmware**: Added `crates/firmware` demo project targeting `thumbv7m-none-eabi`.
- **Core**: Refactored `Machine` to be architecture-agnostic (Pluggable Core).

### Changed
- `labwired-cli` now runs 20,000 steps by default to support firmware boot.
- Updated `docs/architecture.md` and `README.md` with new capabilities.
