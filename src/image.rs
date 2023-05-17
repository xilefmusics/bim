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
            return Err("unsupported png variant".into());
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

    pub fn has_pixel(&self, pixel: &Pixel) -> bool {
        self.pixel_at(pixel.x(), pixel.y())
    }

    pub fn clear_pixel(&mut self, pixel: &Pixel) {
        self.pixel_clear(pixel.x(), pixel.y());
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
}
