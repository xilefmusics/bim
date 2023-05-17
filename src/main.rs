use std::error::Error;
use std::path::Path;

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | (bytes[3] as u32)
}

#[derive(Debug, Clone)]
struct RawChunk<'a> {
    length: u32,
    chunk_type: u32,
    data: &'a [u8],
    crc: u32,
}

impl<'a> RawChunk<'a> {
    pub fn read(bytes: &'a [u8]) -> (Self, usize) {
        let length = bytes_to_u32(&bytes[..4]);
        let chunk_type = bytes_to_u32(&bytes[4..8]);
        let data_end = 8 + length as usize;
        let data = &bytes[8..data_end];
        let crc = bytes_to_u32(&bytes[data_end..data_end + 4]);
        (
            Self {
                length,
                chunk_type,
                data,
                crc,
            },
            data_end + 4,
        )
    }
}

#[derive(Debug, Clone)]
struct Ihdr {
    width: u32,
    height: u32,
    bitdepth: u32,
    color_type: u32,
    compression_method: u32,
    filter_method: u32,
    interlay_method: u32,
}

impl Ihdr {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bitdepth(&self) -> u32 {
        self.bitdepth
    }

    pub fn color_type(&self) -> u32 {
        self.color_type
    }

    pub fn compression_method(&self) -> u32 {
        self.compression_method
    }

    pub fn filter_method(&self) -> u32 {
        self.filter_method
    }

    pub fn interlay_method(&self) -> u32 {
        self.interlay_method
    }

    pub fn from_raw(chunk: RawChunk) -> Result<Self, Box<dyn Error>> {
        if chunk.length != 13 || chunk.data.len() != 13 || chunk.chunk_type != 0x49484452 {
            return Err("not an valid IHDR chunk".into());
        }
        let width = bytes_to_u32(&chunk.data[..4]);
        let height = bytes_to_u32(&chunk.data[4..8]);
        let bitdepth = chunk.data[8] as u32;
        let color_type = chunk.data[9] as u32;
        let compression_method = chunk.data[10] as u32;
        let filter_method = chunk.data[11] as u32;
        let interlay_method = chunk.data[12] as u32;
        Ok(Self {
            width,
            height,
            bitdepth,
            color_type,
            compression_method,
            filter_method,
            interlay_method,
        })
    }
}

#[derive(Debug, Clone)]
struct Phys {
    horizontal_resolution: u32,
    vertical_resolution: u32,
    unit_specifier: u32,
}

impl Phys {
    pub fn from_raw(chunk: RawChunk) -> Result<Self, Box<dyn Error>> {
        if chunk.length != 9 || chunk.data.len() != 9 || chunk.chunk_type != 0x70485973 {
            return Err("not an valid PHYS chunk".into());
        }
        let horizontal_resolution = bytes_to_u32(&chunk.data[..4]);
        let vertical_resolution = bytes_to_u32(&chunk.data[4..8]);
        let unit_specifier = chunk.data[8] as u32;
        Ok(Self {
            horizontal_resolution,
            vertical_resolution,
            unit_specifier,
        })
    }
}

struct ToBoolPalette<'a> {
    palette: Vec<bool>,
    data: &'a [u8],
}

impl<'a> Iterator for ToBoolPalette<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() == 0 {
            return None;
        }
        let data = self.palette[self.data[0] as usize];
        self.data = &self.data[1..];
        Some(data)
    }
}

#[derive(Debug)]
struct ToBoolFrom24Bit<'a> {
    threshold: f64,
    data: &'a [u8],
}

impl<'a> Iterator for ToBoolFrom24Bit<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() == 0 {
            return None;
        }
        let max_brightness = 258.7967349098516;
        let r = self.data[0] as f64;
        let g = self.data[1] as f64;
        let b = self.data[2] as f64;
        self.data = &self.data[3..];
        Some(
            (0.299 * r * r + 0.587 * g * g + 0.144 * b * b).sqrt() / max_brightness
                > self.threshold,
        )
    }
}

#[derive(Debug)]
struct Plte<'a> {
    data: &'a [u8],
}

impl<'a> Plte<'a> {
    pub fn from_raw(chunk: RawChunk<'a>) -> Result<Self, Box<dyn Error>> {
        if chunk.length % 3 != 0 || chunk.data.len() % 3 != 0 || chunk.chunk_type != 0x504C5445 {
            return Err("not an valid PLTE chunk".into());
        }
        let data = chunk.data;
        Ok(Self { data })
    }

    pub fn to_bool(&self, threshold: f64) -> ToBoolFrom24Bit<'a> {
        ToBoolFrom24Bit {
            threshold,
            data: self.data,
        }
    }
}

#[derive(Debug)]
struct Idat<'a> {
    data: &'a [u8],
}

impl<'a> Idat<'a> {
    pub fn to_bool_palette(&self, palette: Vec<bool>) -> ToBoolPalette<'a> {
        ToBoolPalette {
            palette,
            data: self.data,
        }
    }

    pub fn from_raw(chunk: RawChunk<'a>) -> Result<Self, Box<dyn Error>> {
        if chunk.length as usize != chunk.data.len() || chunk.chunk_type != 0x49444154 {
            return Err("not an valid IDAT chunk".into());
        }
        let data = chunk.data;
        Ok(Self { data })
    }
}

#[derive(Debug)]
struct RawChunkIterator<'a> {
    bytes: &'a [u8],
}

impl<'a> RawChunkIterator<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
    pub fn parse(self) -> RawToChunkIterator<'a, Self> {
        RawToChunkIterator { iter: self }
    }
}

impl<'a> Iterator for RawChunkIterator<'a> {
    type Item = RawChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.len() == 0 {
            return None;
        }
        let (chunk, next_byte) = RawChunk::read(&self.bytes);
        self.bytes = &self.bytes[next_byte..];
        Some(chunk)
    }
}

#[derive(Debug)]
struct Iend {}

impl Iend {
    pub fn from_raw(chunk: RawChunk) -> Result<Self, Box<dyn Error>> {
        if chunk.length != 0 || chunk.data.len() != 0 || chunk.chunk_type != 0x49454e44 {
            return Err("not an valid IEND chunk".into());
        }
        Ok(Self {})
    }
}

#[derive(Debug)]
enum Chunk<'a> {
    Ihdr(Ihdr),
    Phys(Phys),
    Plte(Plte<'a>),
    Idat(Idat<'a>),
    Iend(Iend),
}

impl<'a> Chunk<'a> {
    pub fn from_raw(chunk: RawChunk<'a>) -> Result<Self, Box<dyn Error>> {
        match chunk.chunk_type {
            0x49484452 => Ok(Self::Ihdr(Ihdr::from_raw(chunk)?)),
            0x70485973 => Ok(Self::Phys(Phys::from_raw(chunk)?)),
            0x504C5445 => Ok(Self::Plte(Plte::from_raw(chunk)?)),
            0x49444154 => Ok(Self::Idat(Idat::from_raw(chunk)?)),
            0x49454e44 => Ok(Self::Iend(Iend::from_raw(chunk)?)),
            chunk_type => Err(format!("unknown chunk_type ({:#x})", chunk_type).into()),
        }
    }
}

#[derive(Debug)]
struct RawToChunkIterator<'a, I>
where
    I: Iterator<Item = RawChunk<'a>>,
{
    iter: I,
}

impl<'a, I> Iterator for RawToChunkIterator<'a, I>
where
    I: Iterator<Item = RawChunk<'a>>,
{
    type Item = Result<Chunk<'a>, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Chunk::from_raw(self.iter.next()?))
    }
}

struct PngDecoder<'a> {
    chunk_iter: RawToChunkIterator<'a, RawChunkIterator<'a>>,
    current_to_bool_palette: Option<ToBoolPalette<'a>>,
    ihdr_chunk: Option<Ihdr>,
    phys_chunk: Option<Phys>,
    palette: Vec<bool>,
    threshold: f64,
}

impl<'a> Iterator for PngDecoder<'a> {
    type Item = Result<bool, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_to_bool_palette.is_some() {
                let mut current_to_bool_palette = self.current_to_bool_palette.take().unwrap();
                let next = current_to_bool_palette.next();
                self.current_to_bool_palette = Some(current_to_bool_palette);
                match next {
                    Some(bool) => return Some(Ok(bool)),
                    None => {
                        self.current_to_bool_palette = None;
                        continue;
                    }
                }
            }

            match self.next_chunk() {
                Ok(true) => continue,
                Ok(false) => return None,
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

impl<'a> PngDecoder<'a> {
    fn next_chunk(&mut self) -> Result<bool, Box<dyn Error>> {
        let chunk = Some(self.chunk_iter.next().unwrap()?);

        match chunk {
            Some(Chunk::Ihdr(chunk)) => {
                self.ihdr_chunk = Some(chunk);
                Ok(true)
            }
            Some(Chunk::Phys(chunk)) => {
                self.phys_chunk = Some(chunk);
                Ok(true)
            }
            Some(Chunk::Iend(_)) => Ok(false),
            Some(Chunk::Plte(chunk)) => {
                self.palette = chunk.to_bool(self.threshold).collect::<Vec<bool>>();
                Ok(true)
            }
            None => Ok(false),
            Some(Chunk::Idat(chunk)) => {
                if self.ihdr_chunk.is_none() {
                    return Err("Try to parse IDAT chunk before IHDR header read".into());
                }
                let ihdr_chunk = self.ihdr_chunk.as_ref().unwrap();
                if ihdr_chunk.bitdepth() != 8
                    || ihdr_chunk.color_type() != 3
                    || ihdr_chunk.compression_method() != 0
                    || ihdr_chunk.filter_method() != 0
                    || ihdr_chunk.interlay_method() != 0
                {
                    return Err("Unsupported PNG variant".into());
                }

                if self.palette.len() == 0 {
                    return Err("Try to parse IDAT chunk before palette is known".into());
                }

                self.current_to_bool_palette = Some(chunk.to_bool_palette(self.palette.clone()));
                Ok(true)
            }
        }
    }

    pub fn new(data: &'a [u8], threshold: f64) -> Result<Self, Box<dyn Error>> {
        if &data[..8] != &[137, 80, 78, 71, 13, 10, 26, 10] {
            return Err("not a valid png file".into());
        }
        let data = &data[8..];

        let chunk_iter = RawChunkIterator::from_bytes(data).parse();

        let ihdr_chunk = None;
        let phys_chunk = None;
        let palette = vec![];
        let current_to_bool_palette = None;
        let mut decoder = Self {
            chunk_iter,
            ihdr_chunk,
            phys_chunk,
            palette,
            threshold,
            current_to_bool_palette,
        };

        while decoder.current_to_bool_palette.is_none() {
            decoder.next_chunk()?;
        }

        Ok(decoder)
    }

    fn width(&self) -> Result<u32, Box<dyn Error>> {
        match &self.ihdr_chunk {
            Some(chunk) => Ok(chunk.width()),
            None => Err("Try to access width, before IHDR chunk was parsed".into()),
        }
    }

    fn height(&self) -> Result<u32, Box<dyn Error>> {
        match &self.ihdr_chunk {
            Some(chunk) => Ok(chunk.height()),
            None => Err("Try to access height, before IHDR chunk was parsed".into()),
        }
    }
}

fn read_png(path: impl AsRef<Path>, threshold: f64) -> Result<(), Box<dyn Error>> {
    let bytes = std::fs::read(path)?;

    let decoder = PngDecoder::new(&bytes, threshold)?;
    let width = decoder.width()?;
    let height = decoder.height()?;
    let data = decoder.collect::<Result<Vec<bool>, Box<dyn Error>>>()?;

    println!(
        "{:?} {:?}x{:?} {:?}={:?}",
        data,
        width,
        height,
        data.len(),
        width * height
    );

    Ok(())
}

fn main() {
    read_png("./input.png", 0.5).unwrap();
}
