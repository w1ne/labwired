# Contributing to LabWired

Thank you for your interest in contributing to LabWired! We welcome contributions from the community.

## Development Workflow

We follow a **Gitflow** workflow:
- **`main`**: Stable production releases. Do not commit here directly.
- **`develop`**: Integration branch for next release. Open PRs against this branch.
- **`feature/name`**: Your working branch.

### 1. Setup
```bash
git clone https://github.com/w1ne/labwired.git
cd labwired
cargo build
```

### 2. Making Changes
1.  Create a feature branch: `git checkout -b feature/my-feature`
2.  Implement your changes.
3.  Add tests for new functionality.
4.  Verify everything:
    ```bash
    cargo test
    cargo clippy
    cargo fmt --all -- --check
    ```

### 3. Testing with Docker
To ensure your changes work in our CI environment:
```bash
docker build -t labwired-test .
docker run --rm labwired-test
```

### 4. Submitting a Pull Request
- Push your branch to GitHub.
- Open a PR against `develop`.
- Ensure all CI checks pass.
- Request review from a maintainer.

## Coding Standards
- **Style**: We use standard `rustfmt` settings.
- **Linting**: No `clippy` warnings allowed.
- **Documentation**: Public APIs must have doc comments (`///`).

## Reporting Issues
Please open an issue on GitHub describing the bug or feature request clearly.
