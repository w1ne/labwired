# Debugging STM32 Firmware in VS Code

LabWired includes a built-in GDB server that allows you to use standard debugging tools like VS Code to inspect your firmware's execution in real-time.

## Prerequisites

1.  **VS Code** installed.
2.  **Extensions**:
    -   [Cortex-Debug](https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug) (Recommended)
    -   OR [C/C++](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools) (for standard GDB support)
3.  **Toolchain**: `arm-none-eabi-gdb` must be in your PATH.

## Project Configuration

LabWired comes with pre-configured `.vscode` files to get you started immediately.

### launch.json
This file defines how VS Code connects to the LabWired GDB server. It includes configurations for both `Cortex-Debug` and standard `cppdbg`.

### tasks.json
This file defines background tasks, such as starting the LabWired simulator in GDB mode before the debugger attaches.

## Step-by-Step Debugging

1.  **Build your Firmware**:
    Ensure your firmware is built with debug symbols.
    ```bash
    cargo build --target thumbv7m-none-eabi
    ```

2.  **Start Debugging**:
    -   Go to the "Run and Debug" view in VS Code (Ctrl+Shift+D).
    -   Select **"LabWired Debug (Cortex-Debug)"** or **"LabWired Debug (GDB)"** from the dropdown.
    -   Press **F5**.

3.  **What Happens Automatically**:
    -   VS Code runs the "Start LabWired GDB" task.
    -   LabWired starts and waits for a GDB connection on port `3333`.
    -   VS Code attaches to the GDB server, loads the symbols from your ELF, and hits the entry point (or `main`).

## Features Supported

-   **Breakpoints**: Set breakpoints directly in your Rust/C code.
-   **Step Over/Into/Out**: Step through your code instruction-by-instruction or line-by-line.
-   **Variables & Watch**: Inspect local and global variables.
-   **Memory View**: View raw memory at any address (Flash, RAM, or Peripherals).
-   **Peripheral Registers**: (With Cortex-Debug) Use an SVD file to view memory-mapped registers in a structured way.

## Troubleshooting

-   **"Connection Refused"**: Ensure LabWired started successfully. Check the "Terminal" tab in VS Code for any error messages from the "Start LabWired GDB" task.
-   **"Command not found"**: Ensure `arm-none-eabi-gdb` is installed on your system.
-   **Instruction Tracing**: You can enable Instruction Tracing in `tasks.json` by adding `--trace` to the command for extra visibility in the terminal while debugging.
