# Declarative Register Maps (Design)

One of LabWired's core goals is to break the "peripheral modeling bottleneck." Instead of writing manual Rust code for every peripheral, we aim to use declarative YAML specifications to define register maps and behaviors.

## YAML Schema Concept

```yaml
peripheral: "SPI"
version: "1.0"
registers:
  - id: "CR1"
    address_offset: 0x00
    size: 16
    access: "R/W"
    reset_value: 0x0000
    fields:
      - name: "SPE"
        bit_range: [6, 6]
        description: "SPI Enable"
      - name: "MSTR"
        bit_range: [2, 2]
        description: "Master Selection"

  - id: "DR"
    address_offset: 0x0C
    size: 16
    access: "R/W"
    side_effects:
      on_read: "clear_rxne"
      on_write: "start_tx"
```

## Proposed Architecture

1. **Parser**: A Rust crate that reads these YAML files into a structured `PeripheralDescriptor` IR.
2. **Generic Peripheral**: A standard implementation of the `Peripheral` trait that takes a `PeripheralDescriptor` and manages a byte buffer for registers.
3. **Logic Hooks**: A way to attach custom Rust logic (e.g., "start_tx") to specific register offsets defined in the YAML.

## Benefits

- **Consistency**: All peripherals share the same basic MMIO logic (matching addresses, bit masking).
- **Correctness**: Reset values and access permissions are enforced by the generic engine.
- **Speed**: New peripherals can be "modeled" in minutes by copying data from a silicon vendor's SVD file.

## Next Steps

- Implement the `PeripheralDescriptor` parser using `serde_yaml`.
- Develop the `GenericPeripheral` implementation.
- Build a code generator or a dynamic interpreter for these maps.
