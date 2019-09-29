#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::{delay::Delay, prelude::*, spi::Spi, stm32, time::Hertz, timers::Timer};

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

mod keypad;
mod output;
mod random;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
            let game_data = include_bytes!("../../Space Invaders.ch8");

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

            // Display spi pins
            let spi_pins = (
                gpioa.pa5.into_alternate_af0(cs),
                gpioa.pa6.into_alternate_af0(cs),
                gpioa.pa7.into_alternate_af0(cs),
            );
            let (dc, rst, cs) = (
                gpiob.pb0.into_push_pull_output(cs),
                gpiob.pb1.into_push_pull_output(cs),
                gpioa.pa15.into_push_pull_output(cs),
            );
            let spi = Spi::spi1(p.SPI1, spi_pins, ili9341::MODE, Hertz(48_000_000), &mut rcc);
            let mut ili = ili9341::Ili9341::new(spi, cs, dc, rst, &mut delay).unwrap();
            // Check display resolution
            ili.set_orientation(ili9341::Orientation::Landscape)
                .unwrap();
            let mut instruction_timer =
                Timer::tim16(p.TIM16, Hertz(chip8::INSTRUCTION_RATE), &mut rcc);
            let mut delay_timer = Timer::tim17(p.TIM17, Hertz(chip8::TIMER_RATE), &mut rcc);
            let mut computer = chip8::Chip8::new(game_data, random::RandomGen { state: 43 });
            let mut pressed_key = None;
            let mut output = false;
            loop {
                if instruction_timer.wait().is_ok() {
                    computer.run_cycle();
                    let pressed_keys =
                        keypad::read_keypad(&mut r1, &mut r2, &mut r3, &mut r4, &c1, &c2, &c3, &c4)
                            .unwrap();
                    hprintln!("{:?}, {:?}", pressed_keys, c1.is_low()).unwrap();
                    let curr_pressed_key = pressed_keys.trailing_zeros() as u8;
                    // TODO Expand for multiple pressed keys
                    pressed_key = if curr_pressed_key == 16 {
                        if let Some(prev_pressed_key) = pressed_key {
                            computer.handle_key_release(prev_pressed_key);
                        }
                        None
                    } else {
                        if None == pressed_key {
                            computer.handle_key_press(curr_pressed_key);
                        }
                        Some(curr_pressed_key)
                    };
                }
                if delay_timer.wait().is_ok() {
                    computer.timer_tick();
                    output = !output;
                    // Display at 30Hz
                    if output {
                        let buffer = computer.display.get_buffer();
                        let output_iter = output::OutputData::new(&buffer);
                        ili.draw_iter(0, 0, 319, 239, output_iter).unwrap();
                    }
                }
            }
        });
    }

    loop {
        continue;
    }
}
