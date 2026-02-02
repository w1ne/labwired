#![no_main]
#![no_std]

use panic_halt as _;
use cortex_m_rt::entry;

// UART Stub address in Simulator
const UART_TX: *mut u8 = 0x4000_C000 as *mut u8;

#[entry]
fn main() -> ! {
    let message = "Hello, LabWired!\n";

    for byte in message.bytes() {
        unsafe {
            // Write to UART TX register
            // Logic: STR R0, [R1] where R1 = 0x4000_C000
            core::ptr::write_volatile(UART_TX, byte);
        }
    }

    loop {
        // Infinite loop
    }
}
