# LabWired Standalone Simulator - Iteration 1 Plan

## Objective
Deliver a standalone command-line tool (`sim-cli`) capable of loading an ELF binary and executing a basic simulation loop for an ARM Cortex-M architecture (initially mocked/simplified).

## Roadmap

### Phase 1: Foundation (Completed)
- [x] Project Structure (Workspace)
- [x] Release & Merging Strategy Defined (`docs/release_strategy.md`)
- [x] Core Traits (CPU, MemoryBus, Peripheral)
- [x] Error Handling Strategy (`thiserror`)
- [x] Logging/Tracing Setup

### Phase 2: Loader (Completed)
- [x] Integrate `goblin` dependency
- [x] Implement `ElfLoader` struct
- [x] Parse entry point and memory segments from ELF

### Phase 3: Core Simulation Loop (Completed)
- [x] Implement `Cpu` struct (Cortex-M Stub)
- [x] Implement `Memory` struct (Flat byte array)
- [x] Implement `Bus` to route traffic between CPU and Memory
- [x] Basic FE (Fetch-Execute) cycle loop

### Phase 4: CLI & Basic Decoding (Completed)
- [x] Argument parsing (`clap`)
- [x] Connect `loader` output to `core` initialization
- [x] Run the simulation loop
- [x] Implement basic Thumb-2 Decoder (`MOV`, `B`)
- [x] Verify verification with `tests/dummy.elf`

### Phase 5: Verification (Completed)
- [x] Integration tests using a dummy ELF (or just a binary file)
- [x] CI pipeline

### Phase 6: Infrastructure Portability (Completed)
- [x] Dockerfile for testing
- [x] Docker-based verification
