#!/bin/bash
# Script to install git hooks

set -e

HOOKS_DIR=$(git rev-parse --show-toplevel)/.git/hooks
SCRIPTS_DIR=$(git rev-parse --show-toplevel)/scripts

echo "Making scripts executable..."
chmod +x "$SCRIPTS_DIR/pre-commit.sh"

echo "Installing pre-commit hook..."
ln -sf "../../scripts/pre-commit.sh" "$HOOKS_DIR/pre-commit"

echo "Hooks installed successfully!"
