use crate::display::{Buffer, HEIGHT, WIDTH};
pub const DISPLAY_HEIGHT: u32 = 240;
pub const DISPLAY_WIDTH: u32 = 320;
const SCALE_X: u32 = DISPLAY_WIDTH as u32 / WIDTH as u32;
const SCALE_Y: u32 = DISPLAY_HEIGHT as u32 / HEIGHT as u32;
// Maybe use a consisten scale?
//const SCALE: u16 = if SCALE_Y < SCALE_X {SCALE_Y} else {SCALE_X};

pub struct OutputData<'a> {
    buffer: &'a Buffer,
    buffer_pos: (u32, u32),
    count: (u32, u32),
}

impl<'a> OutputData<'a> {
    pub fn new(buffer: &'a Buffer) -> OutputData<'a> {
        OutputData {
            buffer,
            buffer_pos: (0, 0),
            count: (0, 0),
        }
    }
}
impl<'a> Iterator for OutputData<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.0 < SCALE_X {
            self.count.0 += 1;
        } else if self.buffer_pos.0 + 1 < WIDTH as u32 {
            // Begin next x buffer pixel
            self.buffer_pos.0 += 1;
            self.count.0 = 1;
        } else {
            // We're in the x overflow area
            let x_pos = self.buffer_pos.0 * SCALE_X + self.count.0;
            if x_pos + 1 < DISPLAY_HEIGHT {
                self.count.0 += 1;
                return Some(0x0000);
            }
            // Next line
            else if self.count.1 + 1 < SCALE_Y {
                self.buffer_pos.0 = 0;
                self.count.0 = 1;
                self.count.1 += 1;
            } else if self.buffer_pos.1 + 1 < HEIGHT as u32 {
                // Begin next y buffer pixel
                self.buffer_pos.0 = 0;
                self.count.0 = 1;
                self.count.1 = 0;
                self.buffer_pos.1 += 1;
            } else {
                // We're in the y overflow area;
                let overflow_pixels = (DISPLAY_HEIGHT - HEIGHT as u32 * SCALE_Y) * DISPLAY_WIDTH;
                if self.count.1 + 1 < overflow_pixels {
                    self.count.1 += 1;
                    return Some(0x0000);
                } else {
                    return None;
                }
            }
        }
        if self.buffer[self.buffer_pos.1 as usize][self.buffer_pos.0 as usize] {
            Some(0xFFFF)
        } else {
            Some(0x0000)
        }
    }
}
