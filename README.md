# LabWired Firmware Simulation Platform

> A cloud-native, high-performance, standalone firmware simulator for ARM Cortex-M microcontrollers.

## 📖 Overview
LabWired is a next-generation simulation platform designed to bridge the gap between hardware dependency and software velocity. It enables developers to run, debug, and test firmware binaries without physical hardware, leveraging a portable Rust-based execution engine.

**Key Features:**
- **Declarative Configuration**: Define Chips and Boards in YAML (including memory maps and peripherals).
- **System Services**: Full support for SysTick, Vector Table Boot, and Exception Handling.
- **Functional Stubbing**: Mock external sensors and devices without complex emulation.
- **High Performance**: Native Rust implementation (`labwired-core`).
- **Cloud Ready**: Designed for headless execution in CI/CD pipelines.

## 🏗 Architecture
The project is organized as a Rust Workspace:

- **`crates/cli`**: The command-line interface entry point.
- **`crates/config`**: YAML-based hardware and project descriptors.
- **`crates/loader`**: ELF binary parsing and image generation.
- **`crates/core`**: The execution engine (CPU, Dynamic Bus, Peripherals).

See [Architecture Documentation](docs/architecture.md) for details.

## 🚀 Getting Started

### Prerequisites
- **Rust**: Latest stable toolchain (1.75+).
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Building
```bash
# Build all crates
cargo build

# Run tests
cargo test
```

### Running the Simulator (Firmware Mode)

**1. Install ARM Target**
The firmware is built for the `thumbv7m-none-eabi` target (Cortex-M3).
```bash
rustup target add thumbv7m-none-eabi
```

**2. Build the Firmware**
Compile the demo application located in `crates/firmware`.
```bash
cargo build --release -p firmware --target thumbv7m-none-eabi
```

**3. Run the Simulator**
Pass the path to the firmware and the **System Manifest** defining the hardware.
```bash
# Run with prototype STM32F103 configuration
cargo run -p labwired-cli -- --firmware target/thumbv7m-none-eabi/release/firmware --system system.yaml
```

**Expected Output:**
```text
INFO labwired: Starting LabWired Simulator
INFO labwired: Loading system manifest: "system.yaml"
INFO labwired: Loading chip descriptor: "configs/chips/stm32f103.yaml"
INFO labwired: Loading firmware: "..."
Hello, LabWired!
INFO labwired: Simulation loop finished (demo).
```

## 🤝 Development Workflow
We follow **Gitflow** and enforce strict quality gates.

- **Main Branch**: `main` (Production tags only).
- **Development**: `develop` (Feature integration).
- **Feature Branches**: `feature/xyz`.

**Quality Gates:**
- All PRs must pass CI (Format, Lint, Test, Audit).
- Code coverage goal: >80%.

See [Release & Merging Strategy](docs/release_strategy.md) for the full protocol.

## 📄 Documentation
- [Implementation Plan](docs/plan.md)
- [Architecture](docs/architecture.md)
- [Release Strategy](docs/release_strategy.md)

## ⚖️ License
MIT
