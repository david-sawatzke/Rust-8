#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use stm32f1xx_hal as hal;

use crate::hal::{delay::Delay, prelude::*, spi::Spi, stm32, time::Hertz, timer::Timer};
use embedded_hal::digital::v1_compat::OldOutputPin;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

mod keypad;
mod output;
mod random;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        let (
            mut ili,
            mut instruction_timer,
            mut delay_timer,
            (mut r1, mut r2, mut r3, mut r4),
            (c1, c2, c3, c4),
        ) = cortex_m::interrupt::free(move |_cs| {
            let mut flash = p.FLASH.constrain();
            let mut rcc = p.RCC.constrain();
            let mut afio = p.AFIO.constrain(&mut rcc.apb2);

            let clocks = rcc
                .cfgr
                .sysclk(Hertz(64_000_000))
                .pclk1(Hertz(32_000_000))
                .freeze(&mut flash.acr);

            // Get delay provider
            let mut delay = Delay::new(cp.SYST, clocks);

            let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
            let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

            // (Re-)configure the row lines as output
            let row_pins = (
                gpiob.pb12.into_open_drain_output(&mut gpiob.crh),
                gpiob.pb13.into_open_drain_output(&mut gpiob.crh),
                gpiob.pb14.into_open_drain_output(&mut gpiob.crh),
                gpiob.pb15.into_open_drain_output(&mut gpiob.crh),
            );
            // Collums as pull-up input
            // so the input is 0 when button is pressed
            let collum_pins = (
                gpioa.pa8.into_pull_up_input(&mut gpioa.crh),
                gpioa.pa9.into_pull_up_input(&mut gpioa.crh),
                gpioa.pa10.into_pull_up_input(&mut gpioa.crh),
                gpioa.pa11.into_pull_up_input(&mut gpioa.crh),
            );

            // Display spi pins
            let spi_pins = (
                gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
                gpioa.pa6,
                gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
            );
            let (dc, rst, cs) = (
                OldOutputPin::new(gpioa.pa4.into_push_pull_output(&mut gpioa.crl)),
                OldOutputPin::new(gpioa.pa3.into_push_pull_output(&mut gpioa.crl)),
                OldOutputPin::new(gpioa.pa2.into_push_pull_output(&mut gpioa.crl)),
            );
            let spi = Spi::spi1(
                p.SPI1,
                spi_pins,
                &mut afio.mapr,
                ili9341::MODE,
                Hertz(64_000_000),
                clocks,
                &mut rcc.apb2,
            );
            let mut ili = ili9341::Ili9341::new(spi, cs, dc, rst, &mut delay).unwrap();
            // Check display resolution
            ili.set_orientation(ili9341::Orientation::Landscape)
                .unwrap();
            let instruction_timer = Timer::tim3(
                p.TIM3,
                Hertz(chip8::INSTRUCTION_RATE),
                clocks,
                &mut rcc.apb1,
            );
            let delay_timer = Timer::tim4(p.TIM4, Hertz(chip8::TIMER_RATE), clocks, &mut rcc.apb1);
            (ili, instruction_timer, delay_timer, row_pins, collum_pins)
        });
        let game_data = include_bytes!("../../Space Invaders.ch8");
        let mut computer = chip8::Chip8::new(game_data, random::RandomGen { state: 43 });
        let mut pressed_key = None;
        let mut output = false;
        loop {
            // TODO Add counter while the output stuff is running
            if instruction_timer.wait().is_ok() {
                computer.run_cycle();
                let pressed_keys =
                    keypad::read_keypad(&mut r1, &mut r2, &mut r3, &mut r4, &c1, &c2, &c3, &c4)
                        .unwrap();
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
    }

    loop {
        continue;
    }
}
