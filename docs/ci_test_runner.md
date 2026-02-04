# CI Test Runner (`labwired test`)

LabWired provides a CI-friendly runner mode driven by a YAML test script:

```bash
labwired test --script test.yaml
```

You can override script inputs with CLI flags:

```bash
labwired test --firmware path/to/fw.elf --system system.yaml --script test.yaml
```

## Exit Codes

- `0`: pass
- `1`: assertion failure (including timeouts unless explicitly asserted)
- `2`: config/script error (invalid YAML, unknown fields, unsupported schema, invalid limits)
- `3`: simulation/runtime error (memory violations, decode errors) unless explicitly asserted

## Script Schema (v1.0)

```yaml
schema_version: "1.0"
inputs:
  firmware: "relative/or/absolute/path/to/fw.elf"
  system: "optional/path/to/system.yaml"
limits:
  max_steps: 100000
  wall_time_ms: 5000   # optional
assertions:
  - uart_contains: "Hello"
  - uart_regex: "^Hello.*$"
  - expected_stop_reason: max_steps
```

Notes:
- Unknown fields are rejected (script parse/validation returns exit code `2`).
- Relative `inputs.firmware` / `inputs.system` paths are resolved relative to the directory containing the script file.
- CLI flags override script inputs:
  - `--firmware` overrides `inputs.firmware`
  - `--system` overrides `inputs.system`

## Stop Reasons

`expected_stop_reason` supports:
- `max_steps`
- `wall_time`
- `memory_violation`
- `decode_error`
- `halt`

Semantics:
- If the simulator hits `wall_time_ms`, the run **fails** with exit code `1` unless you assert `expected_stop_reason: wall_time`.
- If the simulator hits a runtime error (e.g. `memory_violation`), the run **fails** with exit code `3` unless you assert the matching `expected_stop_reason`.

## Artifacts

Use `--output-dir` to write artifacts:

```bash
labwired test --script test.yaml --output-dir out/artifacts
```

Artifacts:
- `out/artifacts/result.json`: machine-readable summary
- `out/artifacts/uart.log`: captured UART TX bytes
- `out/artifacts/junit.xml`: JUnit XML report (for CI UIs)

Alternatively, you can write JUnit XML to a specific path:

```bash
labwired test --script test.yaml --junit out/junit.xml
```

## GitHub Actions Example

```yaml
- name: Run LabWired tests
  run: |
    cargo build --release
    ./target/release/labwired test --script examples/ci/dummy-max-steps.yaml --output-dir artifacts
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: labwired-artifacts
    path: artifacts
```

