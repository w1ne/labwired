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

## Iteration 3: Firmware Support (Planned)
- [ ] Implement Stack Operations
    - `PUSH`, `POP`
- [ ] Implement Control Flow
    - `BL` (Branch Link), `BX` (Branch Exchange)
- [ ] Implement PC-Relative Load
    - `LDR` (Literal)
- [ ] Firmware Project
    - Create `crates/firmware` using `cortex-m-rt`.
    - Implement "Hello World" application.
- [ ] End-to-End Verification
    - Run compiled firmware in simulator.
