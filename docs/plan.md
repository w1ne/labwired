# LabWired Standalone Simulator - Iteration 1 Plan

## Objective
Deliver a standalone command-line tool (`sim-cli`) capable of loading an ELF binary and executing a basic simulation loop for an ARM Cortex-M architecture (initially mocked/simplified).

## Roadmap

### Phase 1: Foundation (Completed)
- [x] Project Structure (Workspace)
    - **Verified**: `Cargo.toml` workspace defines `core`, `loader`, `cli`.
- [x] Release & Merging Strategy Defined (`docs/release_strategy.md`)
    - **Verified**: Document exists and team follows Gitflow.
- [x] Core Traits (CPU, MemoryBus, Peripheral)
    - **Verified**: `crates/core/src/lib.rs` defines `Cpu` and `Bus` traits.
- [x] Error Handling Strategy (`thiserror`)
    - **Verified**: `SimulationError` enum implemented in `crates/core`.
- [x] Logging/Tracing Setup
    - **Verified**: `cli` initializes `tracing_subscriber`, logs visible in stdout.

### Phase 2: Loader (Completed)
- [x] Integrate `goblin` dependency
    - **Verified**: `crates/loader/Cargo.toml` includes `goblin`.
- [x] Implement `ElfLoader` struct
    - **Verified**: `crates/loader/src/lib.rs` implements `load_elf`.
- [x] Parse entry point and memory segments from ELF
    - **Verified**: `ProgramImage` struct successfully populated in `loader` tests.

### Phase 3: Core Simulation Loop (Completed)
- [x] Implement `Cpu` struct (Cortex-M Stub)
    - **Verified**: `CortexM` struct in `crates/core/src/cpu/mod.rs`.
- [x] Implement `Memory` struct (Flat byte array)
    - **Verified**: `LinearMemory` in `crates/core/src/memory/mod.rs`.
- [x] Implement `Bus` to route traffic between CPU and Memory
    - **Verified**: `SystemBus` routes addresses 0x0... to Flash and 0x2... to RAM.
- [x] Basic FE (Fetch-Execute) cycle loop
    - **Verified**: `Machine::step()` fetches instruction from PC and increments it.

### Phase 4: CLI & Basic Decoding (Completed)
- [x] Argument parsing (`clap`)
    - **Verified**: `labwired-cli --help` works, accepts `-f` argument.
- [x] Connect `loader` output to `core` initialization
    - **Verified**: `cli` correctly passes loaded `ProgramImage` to `Machine::load_firmware`.
- [x] Run the simulation loop
    - **Verified**: CLI runs 10 steps of simulation and prints PC updates.
- [x] Implement basic Thumb-2 Decoder (`MOV`, `B`)
    - **Verified**: `crates/core/src/decoder.rs` correctly decodes opcodes `0x202A` (MOV) and `0xE002` (B).
- [x] Verify verification with `tests/dummy.elf`
    - **Verified**: Real ELF file loaded and executed in `cli`.

### Phase 5: Verification (Completed)
- [x] Integration tests using a dummy ELF (or just a binary file)
    - **Verified**: `crates/core/src/tests.rs` validates CPU logic.
- [x] CI pipeline
    - **Verified**: GitHub Actions (`ci.yml`) builds and tests on push.

### Phase 6: Infrastructure Portability (Completed)
- [x] Dockerfile for testing
    - **Verified**: `Dockerfile` builds `rust:latest` image.
- [x] Docker-based verification
    - **Verified**: `docker run` successfully executes `cargo test` suite (9/9 passed).

## Iteration 2: Expanded Capabilities (Completed)
- [x] Arithmetic & Logic Instructions
    - **Verified**: `ADD`, `SUB`, `CMP`, `AND`, `ORR`, `EOR`, `MVN` implemented and tested.
- [x] Memory Operations
    - **Verified**: `LDR` and `STR` implemented and verified via integration tests.
- [x] Portable Core Architecture
    - **Verified**: `Machine` is generic over `Cpu` trait.
- [x] UART Peripheral
    - **Verified**: Mapped to `0x4000_C000`, writes to stdout.

## Iteration 3: Firmware Support (Completed)
- [x] Implement Stack Operations
    - **Verified**: `PUSH`, `POP` implemented and tested.
- [x] Implement Control Flow
    - **Verified**: `BL`, `BX` and `Bcc` implemented.
- [x] Implement PC-Relative Load
    - **Verified**: `LDR` (Literal) handles constant pools.
- [x] Firmware Project
    - **Verified**: `crates/firmware` builds and runs via correctly configured `link.x`.
- [x] End-to-End Verification
    - **Verified**: Firmware boots and executes in simulator.

## Iteration 4: Advanced Core Support (Completed)
- [x] Implement High Register Operations
    - **Verified**: `MOV` and `CMP` support R8-R15 (including SP, LR, PC).
- [x] Implement Byte-level Memory Access
    - **Verified**: `LDRB`, `STRB` implemented for buffer manipulation.
- [x] Refine 32-bit Instruction Handling
    - **Verified**: Robust 32-bit reassembly for `BL`, `MOVW`, `MOVT`.
- [x] Milestone: "Hello, LabWired!" achieved via UART peripheral.

## Iteration 5: System Services & Exception Handling (Completed)
- [x] Implement Vector Table Boot Logic
    - **Verified**: CPU automatically loads SP/PC from 0x0 on reset.
- [x] Implement SysTick Timer
    - **Verified**: Standard `SYST_*` registers implemented and ticking.
- [x] Implement Basic Exception Entry/Exit
    - **Verified**: Stacking/Unstacking logic allows interrupt handling.

## Iteration 6: Descriptor-Based Configuration (Completed)
- [x] Implement YAML Chip Descriptors
    - **Verified**: `configs/chips/stm32f103.yaml` defines memory mapping and peripherals.
- [x] Implement System Manifests
    - **Verified**: `system.yaml` allows wiring of sensors and devices.
- [x] Dynamic SystemBus
    - **Verified**: Bus auto-configures based on descriptor files.
- [x] Functional Device Stubbing
    - **Verified**: `StubPeripheral` allows modeling external hardware.

## Iteration 7: Stack & Advanced Flow Control (Completed)
- [x] Implement `ADD SP, #imm` and `SUB SP, #imm`.
    - **Verified**: `AddSp`/`SubSp` variants in `decoder.rs` (lines 26-27), execution in `cpu/mod.rs` (lines 248-254), tested in `test_iteration_7_instructions` (lines 401-410).
- [x] Implement `ADD (High Register)` for arbitrary register addition.
    - **Verified**: `AddRegHigh` variant in `decoder.rs` (line 28), execution in `cpu/mod.rs` (line 256), tested with R0+R8 addition (lines 412-418).
- [x] Implement `CPSIE/CPSID` for interrupt enable/disable control.
    - **Verified**: `Cpsie`/`Cpsid` variants in `decoder.rs` (lines 29-30), execution in `cpu/mod.rs` (lines 305-312), tested with primask flag verification (lines 420-431).
- [x] Milestone: Full execution of standard `cortex-m-rt` initialization without warnings.
    - **Verified**: Test suite passes (33/33 tests), no unknown instruction warnings during execution.

## Iteration 8: Real-World Compatibility (Completed)
- [x] Implement Block Memory Operations (`LDM/STM`)
    - **Verified**: `Ldm`/`Stm` variants in `decoder.rs` (lines 45-46), execution in `cpu/mod.rs` (lines 374-397), tested in `test_iteration_8_instructions` with register list {R0-R2} (lines 446-467).
- [x] Implement Halfword Access (`LDRH/STRH`)
    - **Verified**: `LdrhImm`/`StrhImm` variants in `decoder.rs` (lines 47-48), `read_u16`/`write_u16` in `bus/mod.rs`, execution in `cpu/mod.rs` (lines 250-287), tested with 16-bit memory operations (lines 436-445).
- [x] Implement Multiplication (`MUL`)
    - **Verified**: `Mul` variant in `decoder.rs` (line 49), execution in `cpu/mod.rs` (lines 439-457) with N/Z flag updates, tested with 100×2=200 (lines 468-477).
- [x] Implement NVIC (Nested Vectored Interrupt Controller)
    - **Verified**: `nvic.rs` peripheral created (96 lines), ISER/ICER/ISPR/ICPR registers with atomic state, integrated in `SystemBus::tick_peripherals` (lines 158-198), tested in `test_nvic_external_interrupt` (lines 483-512).
- [x] Implement SCB with VTOR (Vector Table relocation)
    - **Verified**: `scb.rs` peripheral created (42 lines), VTOR register at 0xE000ED08, shared atomic state with CPU, exception handler lookup uses VTOR offset (cpu/mod.rs lines 175-180), tested in `test_vtor_relocation` (lines 514-537).
- [x] Two-phase interrupt architecture with NVIC filtering
    - **Verified**: `SystemBus::tick_peripherals` implements pend→signal flow, external IRQs (≥16) filtered by NVIC ISER/ISPR, core exceptions (<16) bypass NVIC (bus/mod.rs lines 158-198).
- [x] Milestone: All 33 tests passing, v0.6.0 released
    - **Verified**: `cargo test` shows 33/33 passing, release tag v0.6.0 created and pushed to GitHub, CHANGELOG.md updated with all features.

## Iteration 9: Real Firmware Integration & Peripheral Ecosystem (In Progress)

### Objectives
Bridge the "peripheral modeling bottleneck" by enabling execution of real-world HAL libraries and expanding the peripheral ecosystem.

### Phase A: HAL Compatibility & Missing Instructions
- [ ] Run STM32 HAL examples (GPIO blink, I2C sensor, SPI flash)
- [ ] Identify and implement missing instructions discovered during execution
  - [x] Division instructions (`SDIV`, `UDIV`)
    - **Verified**: 32-bit SDIV/UDIV decoding and execution in `crates/core/src/cpu/mod.rs`; tests in `crates/core/src/tests.rs` (`test_division_instructions`).
  - [ ] Additional Thumb-2 encodings as needed
  - [ ] Bit manipulation instructions (`BFI`, `UBFX`, etc.)
- [x] Add instruction execution tracing for debugging
  - **Verified**: `--trace` flag enables DEBUG-level instruction logging in `crates/cli/src/main.rs`, with per-step trace in `crates/core/src/cpu/mod.rs`.
- [x] Improve error messages for unknown instructions
  - **Verified**: Unknown instruction logs include PC and opcode in `crates/core/src/cpu/mod.rs`.

### Phase B: Core Peripheral Models
- [ ] Implement GPIO peripheral
  - [ ] Memory-mapped registers (ODR, IDR, MODER, etc.)
  - [ ] Pin state tracking and virtual wiring
- [ ] Implement I2C peripheral (master mode)
  - [ ] Standard I2C protocol state machine
  - [ ] Virtual device attachment API
- [ ] Implement SPI peripheral
  - [ ] Full-duplex communication
  - [ ] Virtual slave device support
- [ ] Implement ADC peripheral (basic)
  - [ ] Single-channel conversion
  - [ ] Configurable virtual input values
- [ ] Implement General Purpose Timers (TIM2/TIM3)
  - [ ] Basic counting modes
  - [ ] Interrupt generation on overflow

### Phase C: Peripheral Architecture & Extensibility
- [x] Design pluggable peripheral API
  - **Verified**: `Peripheral` trait and dynamic dispatch via `SystemBus` in `crates/core/src/lib.rs` and `crates/core/src/bus/mod.rs`.
- [ ] Hot-swappable peripheral models
- [x] Create peripheral descriptor format
  - **Verified**: YAML-based `ChipDescriptor`/`PeripheralConfig` in `crates/config/src/lib.rs`.
- [ ] Register map specifications
- [ ] Document peripheral development guide
  - [ ] Tutorial: Creating custom peripherals
  - [ ] API reference documentation

### Phase D: Developer Experience
- [ ] Create example firmware projects
  - [ ] "Blinky" with GPIO
  - [ ] I2C temperature sensor reader
  - [ ] SPI flash memory interface
- [ ] Add execution visualization
  - [x] Instruction trace logging
    - **Verified**: DEBUG-level trace in `crates/core/src/cpu/mod.rs`, enabled by `--trace` in `crates/cli/src/main.rs`.
  - [ ] Register state snapshots
  - [ ] Memory access history
- [ ] Improve CLI usability
  - [ ] Better error diagnostics
  - [ ] Execution statistics (IPS, cycle count)
  - [ ] Breakpoint support (basic)

### Success Criteria
- [ ] Successfully run unmodified `stm32f1xx-hal` GPIO example
- [ ] Execute I2C communication with virtual sensor
- [ ] Demonstrate SPI flash read/write operations
- [ ] Zero "unknown instruction" warnings for standard HAL usage
- [ ] Documentation: "Getting Started with Real Firmware" guide

### Milestone
**"Real Firmware Ready"**: The simulator can execute production-grade HAL libraries and serve as a viable alternative to physical development boards for early-stage firmware development.

### Delivery Milestones (Concrete)
These milestones break Iteration 9 into shippable increments with explicit acceptance checks.

#### Milestone 9.1: Instruction Compatibility Baseline
- [ ] Run a real HAL-based GPIO "Blinky" firmware through the simulator.
- [ ] Add any missing Thumb-2 encodings discovered during execution (no "unknown instruction" logs).
- [ ] Collect a short instruction trace (first 200 steps) to validate decode/execute flow.

**Acceptance Tests**
- `cargo test` passes.
- `cargo run -p labwired-cli -- --firmware <blinky.elf> --system system.yaml --trace` runs for 1,000+ steps without `Unknown instruction` warnings.

#### Milestone 9.2: GPIO Peripheral MVP
- [ ] Implement GPIO peripheral with a minimal STM32F1-compatible register subset.
- [ ] Support at least `MODER/CRL/CRH`, `IDR`, `ODR`, and `BSRR` semantics.
- [ ] Add virtual pin state tracking (read/write reflecting ODR/IDR behavior).

**Acceptance Tests**
- Unit tests for register read/write semantics.
- Example `blinky` toggles a virtual pin state (verified by log or inspection helper).

#### Milestone 9.3: I2C Peripheral MVP
- [ ] Implement I2C master mode with a simple state machine (start, address, read/write, stop).
- [ ] Define a virtual device attachment API (in-memory mock sensor).

**Acceptance Tests**
- Example firmware performs a read from a virtual sensor and logs a value.

#### Milestone 9.4: SPI Peripheral MVP
- [ ] Implement SPI master mode with full-duplex transfer and basic status flags.
- [ ] Support a virtual SPI flash device with a minimal command set (READ, WRITE).

**Acceptance Tests**
- Example firmware reads and writes a block of data through SPI and verifies contents.

#### Milestone 9.5: Peripheral Extensibility & Docs
- [ ] Document peripheral development guide (API, register mapping, wiring).
- [ ] Create `docs/getting_started_real_firmware.md` walkthrough using GPIO blinky.
- [ ] Extend YAML descriptor format to include register map specs (if needed by GPIO/I2C/SPI).

**Acceptance Tests**
- New doc renders cleanly and aligns with CLI usage in README.
