#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::{delay::Delay, prelude::*, stm32};

use bitflags::bitflags;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2;

bitflags! {
    struct RowKeys: u8  {
        const A = 0b0001;
        const B = 0b0010;
        const C = 0b0100;
        const D = 0b1000;
    }
}

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
                    read_keypad(&mut r1, &mut r2, &mut r3, &mut r4, &c1, &c2, &c3, &c4).unwrap();
                hprintln!("{:?}, {:?}", pressed_keys, c1.is_low()).unwrap();
                delay.delay_ms(1_000_u16);
            }
        });
    }

    loop {
        continue;
    }
}

fn read_keypad<R1, R2, R3, R4, C1, C2, C3, C4, E>(
    r1: &mut R1,
    r2: &mut R2,
    r3: &mut R3,
    r4: &mut R4,
    c1: &C1,
    c2: &C2,
    c3: &C3,
    c4: &C4,
) -> Result<u16, E>
where
    R1: v2::OutputPin<Error = E>,
    R2: v2::OutputPin<Error = E>,
    R3: v2::OutputPin<Error = E>,
    R4: v2::OutputPin<Error = E>,
    C1: v2::InputPin<Error = E>,
    C2: v2::InputPin<Error = E>,
    C3: v2::InputPin<Error = E>,
    C4: v2::InputPin<Error = E>,
{
    r1.set_low()?;
    r2.set_high()?;
    r3.set_high()?;
    r4.set_high()?;
    let row1 = read_row(c1, c2, c3, c4)?.bits();
    r1.set_high()?;
    r2.set_low()?;
    let row2 = read_row(c1, c2, c3, c4)?.bits();
    r2.set_high()?;
    r3.set_low()?;
    let row3 = read_row(c1, c2, c3, c4)?.bits();
    r3.set_high()?;
    r4.set_low()?;
    let row4 = read_row(c1, c2, c3, c4)?.bits();
    r4.set_high()?;
    let pressed_keys = row1 as u16 | (row2 as u16) << 4 | (row3 as u16) << 8 | (row4 as u16) << 12;
    Ok(pressed_keys)
}

fn read_row<C1, C2, C3, C4, E>(c1: &C1, c2: &C2, c3: &C3, c4: &C4) -> Result<RowKeys, E>
where
    C1: v2::InputPin<Error = E>,
    C2: v2::InputPin<Error = E>,
    C3: v2::InputPin<Error = E>,
    C4: v2::InputPin<Error = E>,
{
    let mut pressed_keys = RowKeys::empty();
    if c1.is_low()? {
        pressed_keys.insert(RowKeys::A);
    }
    if c2.is_low()? {
        pressed_keys.insert(RowKeys::B);
    }
    if c3.is_low()? {
        pressed_keys.insert(RowKeys::C);
    }
    if c4.is_low()? {
        pressed_keys.insert(RowKeys::D);
    }
    Ok(pressed_keys)
}
