# LabWired Firmware Simulation Platform

> A cloud-native, high-performance, standalone firmware simulator for ARM Cortex-M microcontrollers.

## 📖 Overview
LabWired is a next-generation simulation platform designed to bridge the gap between hardware dependency and software velocity. It enables developers to run, debug, and test firmware binaries without physical hardware, leveraging a portable Rust-based execution engine.

**Key Features:**
- **Standalone Runner**: No external dependencies/interpreters required.
- **High Performance**: Native Rust implementation (`sim-core`).
- **Cloud Ready**: Designed for headless execution in CI/CD pipelines.
- **Extensible**: Modular architecture separating Loader, Core, and CLI.

## 🏗 Architecture
The project is organized as a Rust Workspace:

- **`crates/cli`**: The command-line interface entry point.
- **`crates/loader`**: ELF binary parsing and image generation.
- **`crates/core`**: The execution engine (CPU, Bus, Peripherals).

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

### Running the Simulator
```bash
# Run the CLI via Cargo
cargo run -p labwired-cli -- --help

# Example usage (future)
# cargo run -p labwired-cli -- -f firmware.elf
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
