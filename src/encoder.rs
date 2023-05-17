pub struct OneBitEncoder<'a> {
    data: &'a [bool],
    idx: usize,
}

impl<'a> OneBitEncoder<'a> {
    pub fn new(data: &'a [bool]) -> Self {
        let idx = 0;
        Self { data, idx }
    }
}

impl<'a> Iterator for OneBitEncoder<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.data.len() {
            return None;
        }

        let idx_max = if self.idx + 8 > self.data.len() {
            self.data.len()
        } else {
            self.idx + 8
        };

        let mut byte = 0;
        for (i, idx) in (self.idx..idx_max).enumerate() {
            if !self.data[idx] {
                byte |= 1 << (7 - i);
            }
        }

        self.idx = idx_max;

        Some(byte)
    }
}
