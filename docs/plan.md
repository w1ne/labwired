# LabWired Standalone Simulator - Iteration 1 Plan

## Objective
Deliver a standalone command-line tool (`sim-cli`) capable of loading an ELF binary and executing a basic simulation loop for an ARM Cortex-M architecture (initially mocked/simplified).

## Roadmap

### Phase 1: Foundation (Current)
- [x] Project Structure (Workspace)
- [x] Release & Merging Strategy Defined (`docs/release_strategy.md`)
- [ ] Core Traits (CPU, MemoryBus, Peripheral)
- [ ] Error Handling Strategy (`thiserror`)
- [ ] Logging/Tracing Setup

### Phase 2: Loader
- [ ] Integrate `goblin` dependency
- [ ] Implement `ElfLoader` struct
- [ ] Parse entry point and memory segments from ELF

### Phase 3: Core Simulation Loop
- [ ] Implement `Cpu` struct (Mock/Stub initially)
- [ ] Implement `Memory` struct (Flat byte array)
- [ ] Implement `Bus` to route traffic between CPU and Memory
- [ ] Basic FE (Fetch-Execute) cycle loop

### Phase 4: CLI
- [ ] Argument parsing (`clap`): input file, verbosity
- [ ] Connect `loader` output to `core` initialization
- [ ] Run the simulation loop
- [ ] Handle `Ctrl+C` for graceful exit

### Phase 5: Verification
- [ ] Integration tests using a dummy ELF (or just a binary file)
- [ ] CI pipeline
