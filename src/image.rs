use crate::decoder::{IndexedDecoder, ThreeByteDecoder};
use crate::encoder::OneBitEncoder;
use crate::object::{Object, Pixel};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Image {
    width: usize,
    height: usize,
    data: Vec<bool>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        let data = vec![false; width * height];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_png(path: impl AsRef<Path>, threshold: f64) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let decoder = png::Decoder::new(file);
        let mut reader = decoder.read_info()?;
        let info = reader.info();

        let width = info.width as usize;
        let height = info.height as usize;
        let mut data: Vec<bool> = Vec::with_capacity(width * height);

        if info.color_type == png::ColorType::Indexed && info.bit_depth == png::BitDepth::Eight {
            let palette = ThreeByteDecoder::new(
                &*info
                    .palette
                    .clone()
                    .ok_or("try to access pallette, but it's not there".to_string())?,
                threshold,
            )
            .collect::<Vec<bool>>();

            while let Some(row) = reader.next_row()? {
                data.extend(IndexedDecoder::new(&palette, row.data()))
            }
        } else {
            return Err(format!(
                "unsupported png variant (color_type={:?}, bit_depth={:?})",
                info.color_type, info.bit_depth,
            )
            .into());
        }

        Ok(Self {
            width,
            height,
            data,
        })
    }

    pub fn to_png(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::One);

        let data = (0..self.height)
            .map(|y| &self.data[y * self.width..(y + 1) * self.width])
            .map(|line| OneBitEncoder::new(line))
            .flatten()
            .collect::<Vec<u8>>();

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&data)?;

        Ok(())
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> bool {
        let idx = y * self.width + x;
        if idx >= self.data.len() {
            return false;
        }
        self.data[idx]
    }

    pub fn pixel_clear(&mut self, x: usize, y: usize) {
        let idx = y * self.width + x;
        if idx >= self.data.len() {
            return;
        }
        self.data[idx] = false;
    }

    pub fn pixel_set(&mut self, x: usize, y: usize) {
        let idx = y * self.width + x;
        if idx >= self.data.len() {
            return;
        }
        self.data[idx] = true;
    }

    pub fn has_pixel(&self, pixel: &Pixel) -> bool {
        self.pixel_at(pixel.x(), pixel.y())
    }

    pub fn clear_pixel(&mut self, pixel: &Pixel) {
        self.pixel_clear(pixel.x(), pixel.y());
    }

    pub fn set_pixel(&mut self, pixel: &Pixel) {
        self.pixel_set(pixel.x(), pixel.y());
    }

    pub fn clear(&mut self, pixels: impl IntoIterator<Item = Pixel>) {
        for pixel in pixels.into_iter() {
            self.clear_pixel(&pixel);
        }
    }

    pub fn clear_border(&mut self) {
        for x in 0..self.width {
            self.clear(Pixel::new(x, 0).connected_grow(self));
            self.clear(Pixel::new(x, self.height - 1).connected_grow(self));
        }
        for y in 0..self.height {
            self.clear(Pixel::new(0, y).connected_grow(self));
            self.clear(Pixel::new(self.width - 1, y).connected_grow(self))
        }
    }

    pub fn is_column_empty(&self, x: usize) -> bool {
        for y in 0..self.height {
            if self.pixel_at(x, y) {
                return false;
            }
        }
        true
    }

    pub fn x_min(&self, skip: bool) -> usize {
        let mut x_min = 0;
        if skip {
            for x in 0..self.width / 3 {
                if self.is_column_empty(x) {
                    x_min = x;
                }
            }
            if x_min > self.width / 3 {
                return self.x_min(false);
            }
        } else {
            while self.is_column_empty(x_min) {
                x_min += 1;
            }
        }
        x_min
    }

    pub fn x_max(&self, skip: bool) -> usize {
        let mut x_max = self.width - 1;
        if skip {
            for x in 0..self.width * 2 / 3 {
                let x = self.width - x;
                if self.is_column_empty(x) {
                    x_max = x;
                }
            }
            if x_max < self.width * 2 / 3 {
                return self.x_max(false);
            }
        } else {
            while self.is_column_empty(x_max) {
                x_max -= 1;
            }
        }
        x_max
    }

    pub fn change_border_width(
        &self,
        current_left: usize,
        new_left: usize,
        current_right: usize,
        new_right: usize,
    ) -> Self {
        let width = self.width - current_left + new_left - current_right + new_right;
        let height = self.height;
        let mut data = Vec::with_capacity(width * height);

        for y in 0..self.height {
            for _ in 0..new_left {
                data.push(false);
            }
            for x in current_left..(self.width - current_right) {
                data.push(self.pixel_at(x, y));
            }
            for _ in 0..new_right {
                data.push(false);
            }
        }

        Self {
            width,
            height,
            data,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_pixel_if_at_least_n_neighbors(&mut self, x: usize, y: usize, n: usize) {
        let mut counter = 0;
        if y > 0 && self.pixel_at(x, y - 1) {
            counter += 1
        };
        if self.pixel_at(x, y + 1) {
            counter += 1
        };
        if x > 0 && self.pixel_at(x - 1, y) {
            counter += 1
        };
        if self.pixel_at(x + 1, y) {
            counter += 1
        };
        if counter >= n {
            self.pixel_set(x, y);
        }
    }

    pub fn clear_pixel_if_at_most_n_neighbors(&mut self, x: usize, y: usize, n: usize) {
        let mut counter = 0;
        if y > 0 && self.pixel_at(x, y - 1) {
            counter += 1
        };
        if self.pixel_at(x, y + 1) {
            counter += 1
        };
        if x > 0 && self.pixel_at(x - 1, y) {
            counter += 1
        };
        if self.pixel_at(x + 1, y) {
            counter += 1
        };
        if counter <= n {
            self.pixel_clear(x, y);
        }
    }

    pub fn merge_grow(&mut self, other: &Self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if other.pixel_at(x, y) && !self.pixel_at(x, y) {
                    self.set_pixel_if_at_least_n_neighbors(x, y, 1);
                }
                if other.pixel_at(self.width - x, self.height - y)
                    && !self.pixel_at(self.width - x, self.height - y)
                {
                    self.set_pixel_if_at_least_n_neighbors(self.width - x, self.height - y, 1);
                }
            }
        }
    }

    pub fn remove_salt_and_pepper(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel_if_at_least_n_neighbors(x, y, 3);
                self.clear_pixel_if_at_most_n_neighbors(x, y, 1);
                self.set_pixel_if_at_least_n_neighbors(self.width - x, self.height - y, 3);
                self.clear_pixel_if_at_most_n_neighbors(self.width - x, self.height - y, 1);
            }
        }
    }
}
