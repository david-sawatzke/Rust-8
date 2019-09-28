use bitflags::bitflags;
use embedded_hal::digital::v2::{InputPin, OutputPin};

bitflags! {
    struct RowKeys: u8  {
        const A = 0b0001;
        const B = 0b0010;
        const C = 0b0100;
        const D = 0b1000;
    }
}

pub fn read_keypad<R1, R2, R3, R4, C1, C2, C3, C4, E>(
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
    R1: OutputPin<Error = E>,
    R2: OutputPin<Error = E>,
    R3: OutputPin<Error = E>,
    R4: OutputPin<Error = E>,
    C1: InputPin<Error = E>,
    C2: InputPin<Error = E>,
    C3: InputPin<Error = E>,
    C4: InputPin<Error = E>,
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
    C1: InputPin<Error = E>,
    C2: InputPin<Error = E>,
    C3: InputPin<Error = E>,
    C4: InputPin<Error = E>,
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
