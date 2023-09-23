pub struct ThreeByteDecoder<'a> {
    data: &'a [u8],
    idx: usize,
    red: f64,
    green: f64,
    blue: f64,
    threshold: f64,
    max_value: f64,
}

impl<'a> ThreeByteDecoder<'a> {
    pub fn new(data: &'a [u8], red: f64, green: f64, blue: f64, threshold: f64) -> Self {
        let idx = 0;
        let max_value = 441.6729559300637;
        Self {
            data,
            idx,
            red,
            green,
            blue,
            threshold,
            max_value,
        }
    }
}

impl<'a> Iterator for ThreeByteDecoder<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx + 3 > self.data.len() {
            return None;
        }

        let r = self.data[self.idx] as f64;
        let g = self.data[self.idx + 1] as f64;
        let b = self.data[self.idx + 2] as f64;

        self.idx += 3;

        Some(
            ((self.red - r) * (self.red - r)
                + (self.green - g) * (self.green - g)
                + (self.blue - b) * (self.blue - b))
                .sqrt()
                / self.max_value
                < self.threshold,
        )
    }
}

pub struct IndexedDecoder<'a> {
    data: &'a [u8],
    idx: usize,
    color_palette: &'a [bool],
}

impl<'a> IndexedDecoder<'a> {
    pub fn new(color_palette: &'a [bool], data: &'a [u8]) -> Self {
        let idx = 0;
        Self {
            data,
            idx,
            color_palette,
        }
    }
}

impl<'a> Iterator for IndexedDecoder<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.data.len() {
            return None;
        }

        let b = self.color_palette[self.data[self.idx] as usize];
        self.idx += 1;
        Some(b)
    }
}
