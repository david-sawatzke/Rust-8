use random_trait::Random;

pub struct RandomGen {
    pub state: u32,
}

impl Random for RandomGen {
    type Error = ();
    fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        for e in buf.iter_mut() {
            // Basic xorshift, taken from https://en.wikipedia.org/wiki/Xorshift
            let mut x = self.state;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.state = x;
            *e = x as u8;
        }
        Ok(())
    }
}
