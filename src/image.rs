use crate::cutout::Cutout;
use crate::decoder::{IndexedDecoder, ThreeByteDecoder};
use crate::encoder::OneBitEncoder;
use derivative::Derivative;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Derivative)]
#[derivative(Debug)]
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

    pub fn and(&self, other: &Self) -> Result<Self, String> {
        if self.width != other.width || self.height != other.height {
            return Err("dimensions do not match".into());
        }
        let mut new = Image::new_empty(self.width, self.height);
        for i in 0..self.data.len() {
            new.data[i] = self.data[i] && other.data[i];
        }
        Ok(new)
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
}
