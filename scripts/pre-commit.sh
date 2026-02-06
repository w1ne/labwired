#!/bin/bash
# Pre-commit hook to run linting and validations

set -e

echo "--- [Running Pre-commit Validations] ---"

echo "Checking formatting..."
cargo fmt --all -- --check

echo "Running Clippy (Host)..."
cargo clippy --workspace --exclude firmware --exclude firmware-ci-fixture -- -D warnings

echo "Running Cargo Check (Host)..."
cargo check --workspace --exclude firmware --exclude firmware-ci-fixture

# Optionally run firmware checks if needed, but these might be slow
# echo "Running Clippy (Firmware)..."
# cargo clippy -p firmware --target thumbv7m-none-eabi -- -D warnings

echo "--- [All Validations Passed!] ---"
exit 0
