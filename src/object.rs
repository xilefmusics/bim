use crate::image::Image;
use std::collections::HashSet;

pub trait Object: IntoIterator<Item = Pixel> + Clone {
    fn xmin(&self) -> usize;
    fn xmax(&self) -> usize;
    fn ymin(&self) -> usize;
    fn ymax(&self) -> usize;
    fn width(&self) -> usize {
        self.xmax() - self.xmin() + 1
    }
    fn height(&self) -> usize {
        self.ymax() - self.ymin() + 1
    }
    fn size(&self) -> usize;
    fn touches_border(&self, width: usize, height: usize, offx: usize, offy: usize) -> bool {
        self.xmin() == offx
            || self.ymin() == offy
            || self.xmax() == offx + width - 1
            || self.ymax() == offy + height - 1
    }
    fn to_simple_object(self) -> SimpleObject {
        let mut obj = SimpleObject::new();
        for pixel in self.into_iter() {
            obj.add_pixel(pixel);
        }
        obj
    }
    fn to_image(self) -> Image {
        let width = self.width();
        let height = self.height();
        let xmin = self.xmin();
        let ymin = self.ymin();
        let mut image = Image::new_empty(width, height);
        image.set_pixels(self.into_iter().map(|pixel| pixel.subx(xmin).suby(ymin)));
        image
    }
    fn connected_grow(&self, image: &Image, max_size: Option<usize>) -> Option<SimpleObject> {
        let mut queue = self.clone().into_iter().collect::<Vec<Pixel>>();
        let mut result = SimpleObject::new();

        while let Some(pixel) = queue.pop() {
            if let Some(max_size) = max_size {
                if result.size > max_size {
                    return None;
                }
            }
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

        Some(result)
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

#[derive(Debug, Clone)]
pub struct SimpleObject {
    pixel: HashSet<Pixel>,
    xmin: usize,
    xmax: usize,
    ymin: usize,
    ymax: usize,
    size: usize,
}

impl Object for Pixel {
    fn xmin(&self) -> usize {
        self.x
    }
    fn xmax(&self) -> usize {
        self.x
    }
    fn ymin(&self) -> usize {
        self.y
    }
    fn ymax(&self) -> usize {
        self.y
    }
    fn width(&self) -> usize {
        1
    }
    fn height(&self) -> usize {
        1
    }
    fn size(&self) -> usize {
        1
    }
}

impl SimpleObject {
    pub fn new() -> Self {
        let pixel = HashSet::new();
        let xmin = std::usize::MAX;
        let xmax = 0;
        let ymin = std::usize::MAX;
        let ymax = 0;
        let size = 0;
        Self {
            pixel,
            xmin,
            xmax,
            ymin,
            ymax,
            size,
        }
    }

    pub fn add_pixel(&mut self, pixel: Pixel) -> bool {
        if pixel.x > self.xmax {
            self.xmax = pixel.x;
        }
        if pixel.x < self.xmin {
            self.xmin = pixel.x;
        }
        if pixel.y > self.ymax {
            self.ymax = pixel.y;
        }
        if pixel.y < self.ymin {
            self.ymin = pixel.y;
        }
        self.size = self.size + 1;
        self.pixel.insert(pixel)
    }

    pub fn xmin(&self) -> usize {
        self.xmin
    }

    pub fn xmax(&self) -> usize {
        self.xmax
    }

    pub fn ymin(&self) -> usize {
        self.ymin
    }

    pub fn ymax(&self) -> usize {
        self.ymax
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn has_pixel(&self, pixel: &Pixel) -> bool {
        self.pixel.contains(pixel)
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

impl Object for SimpleObject {
    fn xmin(&self) -> usize {
        self.xmin
    }
    fn xmax(&self) -> usize {
        self.xmax
    }
    fn ymin(&self) -> usize {
        self.ymin
    }
    fn ymax(&self) -> usize {
        self.ymax
    }
    fn width(&self) -> usize {
        self.xmax - self.xmin + 1
    }
    fn height(&self) -> usize {
        self.ymax - self.ymin + 1
    }
    fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rectangle {
    xmin: usize,
    xmax: usize,
    ymin: usize,
    ymax: usize,
}

impl Rectangle {
    pub fn new(xmin: usize, xmax: usize, ymin: usize, ymax: usize) -> Self {
        return Self {
            xmin,
            xmax,
            ymin,
            ymax,
        };
    }
}

impl IntoIterator for Rectangle {
    type Item = Pixel;
    type IntoIter = RectanglePixelIterator;

    fn into_iter(self) -> Self::IntoIter {
        RectanglePixelIterator {
            xmin: self.xmin,
            x: self.xmin,
            xmax: self.xmax,
            y: self.ymin,
            ymax: self.ymax,
            finished: false,
        }
    }
}

pub struct RectanglePixelIterator {
    xmin: usize,
    x: usize,
    xmax: usize,
    y: usize,
    ymax: usize,
    finished: bool,
}

impl Iterator for RectanglePixelIterator {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let pixel = Pixel::new(self.x, self.y);
        self.x = self.x + 1;
        if self.x > self.xmax {
            self.x = self.xmin;
            self.y = self.y + 1;
        }
        if self.y > self.ymax {
            self.finished = true;
        }
        Some(pixel)
    }
}

impl Object for Rectangle {
    fn xmin(&self) -> usize {
        self.xmin
    }
    fn xmax(&self) -> usize {
        self.xmax
    }
    fn ymin(&self) -> usize {
        self.ymin
    }
    fn ymax(&self) -> usize {
        self.ymax
    }
    fn width(&self) -> usize {
        self.xmax - self.xmin + 1
    }
    fn height(&self) -> usize {
        self.ymax - self.ymin + 1
    }
    fn size(&self) -> usize {
        self.height() * self.width()
    }
}

pub struct PixelsToOneHeightRectangles<I>
where
    I: Iterator<Item = Pixel>,
{
    iter: I,
    xmin: usize,
    ymin: usize,
    finished: bool,
}

impl<I> Iterator for PixelsToOneHeightRectangles<I>
where
    I: Iterator<Item = Pixel>,
{
    type Item = Rectangle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let mut lastx = self.xmin;
        while let Some(pixel) = self.iter.next() {
            if pixel.x > lastx + 1 || pixel.x < lastx {
                let rectangle = Rectangle::new(self.xmin, lastx, self.ymin, self.ymin);
                self.xmin = pixel.x;
                self.ymin = pixel.y;
                return Some(rectangle);
            }
            lastx = pixel.x;
        }
        self.finished = true;
        Some(Rectangle::new(self.xmin, lastx, self.ymin, self.ymin))
    }
}

pub trait IntoPixelsToOneHeightRectangles: Iterator {
    fn to_one_height_rectangles(mut self) -> PixelsToOneHeightRectangles<Self>
    where
        Self: Sized + Iterator<Item = Pixel>,
    {
        if let Some(pixel) = self.next() {
            PixelsToOneHeightRectangles {
                xmin: pixel.x(),
                ymin: pixel.y(),
                finished: false,
                iter: self,
            }
        } else {
            PixelsToOneHeightRectangles {
                xmin: 0,
                ymin: 0,
                finished: true,
                iter: self,
            }
        }
    }
}

impl<I> IntoPixelsToOneHeightRectangles for I where I: Sized + Iterator<Item = Pixel> {}

#[derive(Debug, Clone)]
pub struct RectangleCollection {
    rectangles: Vec<Rectangle>,
    bottom_xmin: usize,
    bottom_xmax: usize,
    last_bottom_xmin: usize,
    last_bottom_xmax: usize,
    xmin: usize,
    xmax: usize,
    ymin: usize,
    ymax: usize,
    size: usize,
}
impl RectangleCollection {
    pub fn new(rectangle: Rectangle) -> Self {
        Self {
            xmin: rectangle.xmin,
            xmax: rectangle.xmax,
            ymin: rectangle.ymin,
            ymax: rectangle.ymax,
            bottom_xmin: rectangle.xmin,
            bottom_xmax: rectangle.xmax,
            last_bottom_xmin: rectangle.xmin,
            last_bottom_xmax: rectangle.xmax,
            size: rectangle.size(),
            rectangles: vec![rectangle],
        }
    }

    pub fn add_rectangle(&mut self, rectangle: Rectangle) {
        if rectangle.ymin == self.ymax {
            if rectangle.xmin < self.last_bottom_xmin {
                self.last_bottom_xmin = rectangle.xmin;
            }
            if rectangle.xmax > self.last_bottom_xmax {
                self.last_bottom_xmax = rectangle.xmax;
            }
        } else if rectangle.ymin > self.ymax {
            self.last_bottom_xmax = self.bottom_xmax;
            self.last_bottom_xmin = self.bottom_xmin;
            self.bottom_xmax = self.xmax;
            self.bottom_xmin = self.xmin;
        }
        if rectangle.ymin < self.ymin {
            self.ymin = rectangle.ymin;
        }
        if rectangle.ymax > self.ymax {
            self.ymax = rectangle.ymax;
        }
        if rectangle.xmin < self.xmin {
            self.xmin = rectangle.xmin;
        }
        if rectangle.xmax > self.xmax {
            self.xmax = rectangle.xmax;
        }
        self.size += rectangle.size(); // TODO only true if not overlapping
        self.rectangles.push(rectangle);
    }

    pub fn bottom_touch(&self, rectangle: &Rectangle) -> bool {
        (self.ymax == rectangle.ymin
            && rectangle.xmin <= self.last_bottom_xmax
            && self.last_bottom_xmin <= rectangle.xmax)
            || (self.ymax + 1 == rectangle.ymin
                && rectangle.xmin <= self.bottom_xmax
                && self.bottom_xmin <= rectangle.xmax)
    }

    pub fn rectangles(self) -> Vec<Rectangle> {
        self.rectangles
    }

    pub fn object(self) -> SimpleObject {
        let mut obj = SimpleObject::new();
        for rectangle in self.rectangles {
            for pixel in rectangle {
                obj.add_pixel(pixel);
            }
        }
        obj
    }
}

impl IntoIterator for RectangleCollection {
    type Item = Pixel;
    type IntoIter = RectangleCollectionPixelIterator;

    fn into_iter(self) -> Self::IntoIter {
        let mut outher = self.rectangles().into_iter();
        let inner = outher.next().unwrap().into_iter();
        RectangleCollectionPixelIterator { outher, inner }
    }
}

pub struct RectangleCollectionPixelIterator {
    outher: std::vec::IntoIter<Rectangle>,
    inner: RectanglePixelIterator,
}

impl Iterator for RectangleCollectionPixelIterator {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(pixel) = self.inner.next() {
                return Some(pixel);
            }
            self.inner = self.outher.next()?.into_iter();
        }
    }
}

impl Object for RectangleCollection {
    fn xmin(&self) -> usize {
        self.xmin
    }
    fn xmax(&self) -> usize {
        self.xmax
    }
    fn ymin(&self) -> usize {
        self.ymin
    }
    fn ymax(&self) -> usize {
        self.ymax
    }
    fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectanglePixelIterator() {
        let rectangle = Rectangle::new(0, 0, 0, 0);
        let pixels = vec![Pixel::new(0, 0)];
        assert_eq!(rectangle.into_iter().collect::<Vec<Pixel>>(), pixels)
    }

    #[test]
    fn pixelsToOneHeightRectangles() {
        let pixels = vec![
            Pixel::new(0, 0),
            Pixel::new(1, 0),
            Pixel::new(3, 0),
            Pixel::new(4, 0),
            Pixel::new(5, 0),
            Pixel::new(2, 1),
        ];
        let rectangles = vec![
            Rectangle::new(0, 1, 0, 0),
            Rectangle::new(3, 5, 0, 0),
            Rectangle::new(2, 2, 1, 1),
        ];
        assert_eq!(
            pixels
                .into_iter()
                .to_one_height_rectangles()
                .collect::<Vec<Rectangle>>(),
            rectangles
        )
    }
}
