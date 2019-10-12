#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use stm32f1xx_hal as hal;

use crate::hal::{
    delay::Delay,
    prelude::*,
    spi::Spi,
    stm32,
    stm32::{interrupt, Interrupt, Peripherals, EXTI, TIM3, TIM4},
    time::Hertz,
    timer::Event,
    timer::Timer,
};
use embedded_hal::digital::v1_compat::OldOutputPin;

use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

mod keypad;
mod random;

static INSTRUCTION_COUNTER: Mutex<RefCell<(u32, Option<Timer<TIM3>>)>> =
    Mutex::new(RefCell::new((0, None)));
static DELAY_COUNTER: Mutex<RefCell<(u32, Option<Timer<TIM4>>)>> =
    Mutex::new(RefCell::new((0, None)));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        let (mut ili, (mut r1, mut r2, mut r3, mut r4), (c1, c2, c3, c4), mut delay) =
            cortex_m::interrupt::free(move |cs| {
                let mut flash = p.FLASH.constrain();
                let mut rcc = p.RCC.constrain();
                let mut afio = p.AFIO.constrain(&mut rcc.apb2);
                let mut nvic = cp.NVIC;
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
                    gpioa.pa12.into_pull_up_input(&mut gpioa.crh),
                );

                // Display spi pins
                let spi_pins = (
                    gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
                    gpioa.pa6,
                    gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
                );
                let (dc, rst, dcs) = (
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
                let mut ili = ili9341::Ili9341::new(spi, dcs, dc, rst, &mut delay).unwrap();
                // Check display resolution
                ili.set_orientation(ili9341::Orientation::Landscape)
                    .unwrap();
                let mut instruction_timer = Timer::tim3(
                    p.TIM3,
                    Hertz(chip8::INSTRUCTION_RATE),
                    clocks,
                    &mut rcc.apb1,
                );
                instruction_timer.listen(Event::Update);
                let mut delay_timer =
                    Timer::tim4(p.TIM4, Hertz(chip8::TIMER_RATE), clocks, &mut rcc.apb1);
                delay_timer.listen(Event::Update);
                nvic.enable(Interrupt::TIM3);
                nvic.enable(Interrupt::TIM4);
                INSTRUCTION_COUNTER.borrow(cs).borrow_mut().1 = Some(instruction_timer);
                DELAY_COUNTER.borrow(cs).borrow_mut().1 = Some(delay_timer);

                (ili, row_pins, collum_pins, delay)
            });
        let game_data = include_bytes!("../../Space Invaders.ch8");
        let mut computer = chip8::Chip8::new(game_data, random::RandomGen { state: 43 });
        let mut pressed_key = None;
        loop {
            let (instructions, delays) = free(|cs| {
                let mut instruction_cell = INSTRUCTION_COUNTER.borrow(cs).borrow_mut();
                let mut delay_cell = DELAY_COUNTER.borrow(cs).borrow_mut();
                let counts = (instruction_cell.0, delay_cell.0);
                instruction_cell.0 = 0;
                delay_cell.0 = 0;
                counts
            });

            for _ in 0..instructions {
                computer.run_cycle();
            }
            for _ in 0..delays {
                computer.timer_tick();
            }
            let buffer = computer.display.get_buffer();
            let output_iter = chip8::output::OutputData::new(&buffer);
            ili.draw_iter(0, 0, 319, 239, output_iter).unwrap();
            let pressed_keys = keypad::read_keypad(
                &mut delay, &mut r1, &mut r2, &mut r3, &mut r4, &c1, &c2, &c3, &c4,
            )
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
    }

    loop {
        continue;
    }
}
// Instruction
#[interrupt]
fn TIM3() {
    free(|cs| {
        let mut cell = INSTRUCTION_COUNTER.borrow(cs).borrow_mut();
        cell.0 += 1;
        cell.1.as_mut().unwrap().clear_update_interrupt_flag();
    });
}

// Delay
#[interrupt]
fn TIM4() {
    free(|cs| {
        let mut cell = DELAY_COUNTER.borrow(cs).borrow_mut();
        cell.0 += 1;
        cell.1.as_mut().unwrap().clear_update_interrupt_flag();
    });
}
