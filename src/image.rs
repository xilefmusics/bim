use crate::cutout::Cutout;
use crate::decoder::{IndexedDecoder, ThreeByteDecoder};
use crate::encoder::OneBitEncoder;
use crate::object::Pixel;
use derivative::Derivative;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct Image {
    width: usize,
    height: usize,
    #[derivative(Debug = "ignore")]
    data: Vec<bool>,
}

impl Image {
    pub fn new_empty(width: usize, height: usize) -> Self {
        let data = vec![false; width * height];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn new(width: usize, height: usize, data: Vec<bool>) -> Self {
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

    pub fn has_pixel(&self, pixel: &Pixel) -> bool {
        self.get(pixel.x(), pixel.y())
    }

    pub fn from_png(
        path: impl AsRef<Path>,
        red: f64,
        green: f64,
        blue: f64,
        threshold: f64,
    ) -> Result<Self, Box<dyn Error>> {
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
                red,
                green,
                blue,
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

    pub fn get(&self, x: usize, y: usize) -> bool {
        let idx = y * self.width + x;
        if idx >= self.data.len() {
            return false;
        }
        self.data[idx]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let idx = y * self.width + x;
        if idx >= self.data.len() {
            return;
        }
        self.data[idx] = value;
    }

    pub fn full_cutout(&self) -> Cutout {
        Cutout::new(self, self.width, self.height, 0, 0)
    }

    pub fn cutout(
        &self,
        width: usize,
        height: usize,
        offx: usize,
        offy: usize,
    ) -> Result<Cutout, String> {
        if self.width < offx + width || self.height < offy + height {
            return Err("dimensions do not match".into());
        }
        Ok(Cutout::new(self, width, height, offx, offy))
    }

    pub fn fill_border(&mut self) {
        for x in 0..self.width {
            let mut y = 0;
            while !self.get(x, y) {
                self.set(x, y, true);
                y = y + 1;
            }
            let mut y = self.height - 1;
            while !self.get(x, y) {
                self.set(x, y, true);
                y = y - 1;
            }
        }
        for y in 0..self.height {
            let mut x = 0;
            while !self.get(x, y) {
                self.set(x, y, true);
                x = x + 1;
            }
            let mut x = self.width - 1;
            while !self.get(x, y) {
                self.set(x, y, true);
                x = x - 1;
            }
        }
    }

    pub fn overwrite(&mut self, other: &Image, offx: usize, offy: usize) {
        for y in 0..other.height {
            for x in 0..other.width {
                self.set(x + offx, y + offy, other.get(x, y))
            }
        }
    }

    pub fn set_pixels<T>(&mut self, pixels: T)
    where
        T: IntoIterator<Item = Pixel>,
    {
        for pixel in pixels {
            self.set(pixel.x(), pixel.y(), true)
        }
    }

    pub fn clear_pixels<T>(&mut self, pixels: T)
    where
        T: IntoIterator<Item = Pixel>,
    {
        for pixel in pixels.into_iter() {
            self.set(pixel.x(), pixel.y(), false)
        }
    }

    pub fn diff_down_up(&self) -> Self {
        let mut result = Self::new_empty(self.width, self.height);
        for y in 1..self.height {
            for x in 0..self.width {
                if !self.get(x, y) && self.get(x, y - 1) {
                    result.set(x, y - 1, true)
                }
            }
        }
        result
    }
}
