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
    // 1. Test division operations
    let a: u32 = 100;
    let b: u32 = 5;
    let c: u32 = a / b;

    // 2. Mock DMA configuration (Memory-to-Memory)
    let dma1_base = 0x4002_0000;
    let ch1_ccr = dma1_base + 0x08;
    let ch1_cndtr = dma1_base + 0x0C;
    let ch1_cpar = dma1_base + 0x10;
    let ch1_cmar = dma1_base + 0x14;

    unsafe {
        core::ptr::write_volatile(ch1_cpar as *mut u32, 0x2000_1000); // Src
        core::ptr::write_volatile(ch1_cmar as *mut u32, 0x2000_2000); // Dst
        core::ptr::write_volatile(ch1_cndtr as *mut u32, 8); // 8 bytes
        // Enable DMA: MEM2MEM=1, MINC=1, PINC=1, EN=1
        core::ptr::write_volatile(ch1_ccr as *mut u32, (1 << 14) | (1 << 7) | (1 << 6) | (1 << 0));
    }

    // 3. Mock EXTI configuration
    let exti_base = 0x4001_0400;
    let exti_imr = exti_base + 0x00;
    let exti_swier = exti_base + 0x10;

    unsafe {
        core::ptr::write_volatile(exti_imr as *mut u32, 1 << 0); // Unmask Line 0
        core::ptr::write_volatile(exti_swier as *mut u32, 1 << 0); // Trigger software interrupt
    }

    // Use results to prevent optimization
    unsafe {
        core::ptr::write_volatile(0x2000_0000 as *mut u32, c);
    }

    loop {}
}
