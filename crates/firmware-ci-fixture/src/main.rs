#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

// Matches `SystemBus::new()` default `uart1` base.
const UART_TX_PTR: *mut u8 = 0x4000_C000 as *mut u8;

#[entry]
fn main() -> ! {
    unsafe {
        core::ptr::write_volatile(UART_TX_PTR, b'O');
        core::ptr::write_volatile(UART_TX_PTR, b'K');
        core::ptr::write_volatile(UART_TX_PTR, b'\n');
    }

    // Deterministic "PC stuck" for `no_progress` tests.
    loop {}
}
