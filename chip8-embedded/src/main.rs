#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::{delay::Delay, prelude::*, stm32};

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

mod keypad;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

            // Get delay provider
            let mut delay = Delay::new(cp.SYST, &rcc);

            let gpioa = p.GPIOA.split(&mut rcc);
            let gpiob = p.GPIOB.split(&mut rcc);
            let gpiof = p.GPIOF.split(&mut rcc);

            // (Re-)configure the row lines as output
            let (mut r1, mut r2, mut r3, mut r4) = (
                gpiob.pb8.into_open_drain_output(cs),
                gpiof.pf0.into_open_drain_output(cs),
                gpiof.pf1.into_open_drain_output(cs),
                gpioa.pa0.into_open_drain_output(cs),
            );
            // Collums as pull-up input
            // so the input is 0 when button is pressed
            let (c1, c2, c3, c4) = (
                gpioa.pa1.into_pull_up_input(cs),
                gpioa.pa2.into_pull_up_input(cs),
                gpioa.pa3.into_pull_up_input(cs),
                gpioa.pa4.into_pull_up_input(cs),
            );

            loop {
                let pressed_keys =
                    keypad::read_keypad(&mut r1, &mut r2, &mut r3, &mut r4, &c1, &c2, &c3, &c4)
                        .unwrap();
                hprintln!("{:?}, {:?}", pressed_keys, c1.is_low()).unwrap();
                delay.delay_ms(1_000_u16);
            }
        });
    }

    loop {
        continue;
    }
}
