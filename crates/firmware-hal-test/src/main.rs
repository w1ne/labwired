#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // Get access to device peripherals
    let dp = pac::Peripherals::take().unwrap();
    
    // Configure the clock
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    
    // Configure GPIO pin C13 as output (LED on Blue Pill)
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    
    // Simple blink loop
    loop {
        led.set_high();
        cortex_m::asm::delay(clocks.sysclk().raw() / 4);
        led.set_low();
        cortex_m::asm::delay(clocks.sysclk().raw() / 4);
    }
}
