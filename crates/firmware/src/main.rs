#![no_std]
// LabWired - Firmware Simulation Platform
// Copyright (C) 2026 Andrii Shylenko
//
// This software is released under the MIT License.
// See the LICENSE file in the project root for full license information.

#![no_main]
#![allow(clippy::empty_loop)]

use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Test division operations
    let a: u32 = 100;
    let b: u32 = 5;
    let c: u32 = a / b; // Should trigger UDIV instruction

    let d: i32 = -100;
    let e: i32 = 5;
    let f: i32 = d / e; // Should trigger SDIV instruction

    // Use the results to prevent optimization
    unsafe {
        core::ptr::write_volatile(0x2000_0000 as *mut u32, c);
        core::ptr::write_volatile(0x2000_0004 as *mut i32, f);
    }

    loop {}
}
