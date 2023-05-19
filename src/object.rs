use crate::image::Image;
use std::collections::HashSet;

pub trait Object: IntoIterator<Item = Pixel> + Clone {
    fn connected_grow(&self, image: &Image) -> SimpleObject {
        let mut queue = self.clone().into_iter().collect::<Vec<Pixel>>();
        let mut result = SimpleObject::new();

        while let Some(pixel) = queue.pop() {
            if !image.has_pixel(&pixel) {
                continue;
            }
            if !result.add_pixel(pixel.clone()) {
                continue;
            }
            if pixel.x < image.width() {
                queue.push(pixel.addx(1));
            }
            if pixel.y < image.height() {
                queue.push(pixel.addy(1));
            }
            if pixel.x > 0 {
                queue.push(pixel.subx(1));
            }
            if pixel.y > 0 {
                queue.push(pixel.suby(1));
            }
        }

        result
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Pixel {
    x: usize,
    y: usize,
}

impl Pixel {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn addx(&self, x: usize) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
        }
    }

    pub fn subx(&self, x: usize) -> Self {
        Self {
            x: self.x - x,
            y: self.y,
        }
    }

    pub fn addy(&self, y: usize) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
        }
    }

    pub fn suby(&self, y: usize) -> Self {
        Self {
            x: self.x,
            y: self.y - y,
        }
    }
}

impl IntoIterator for Pixel {
    type Item = Pixel;
    type IntoIter = std::iter::Once<Self>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl Object for Pixel {}

#[derive(Debug, Clone)]
pub struct SimpleObject {
    pixel: HashSet<Pixel>,
}

impl SimpleObject {
    pub fn new() -> Self {
        let pixel = HashSet::new();
        Self { pixel }
    }

    pub fn add_pixel(&mut self, pixel: Pixel) -> bool {
        self.pixel.insert(pixel)
    }
}

impl IntoIterator for SimpleObject {
    type Item = Pixel;
    type IntoIter = std::collections::hash_set::IntoIter<Pixel>;

    fn into_iter(self) -> Self::IntoIter {
        self.pixel.into_iter()
    }
}

impl FromIterator<Pixel> for SimpleObject {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Pixel>,
    {
        let mut iter = iter.into_iter();
        let mut result = Self::new();
        while let Some(pixel) = iter.next() {
            result.add_pixel(pixel);
        }
        result
    }
}

impl Object for SimpleObject {}
