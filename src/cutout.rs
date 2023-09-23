use crate::image::Image;
use crate::object::{IntoPixelsToOneHeightRectangles, Object, Pixel, RectangleCollection};

#[derive(Clone, Debug)]
pub struct Cutout<'a> {
    image: &'a Image,
    width: usize,
    height: usize,
    offx: usize,
    offy: usize,
}

impl<'a> Cutout<'a> {
    pub fn new(image: &'a Image, width: usize, height: usize, offx: usize, offy: usize) -> Self {
        Self {
            image,
            width,
            height,
            offx,
            offy,
        }
    }

    pub fn image(&self) -> &Image {
        self.image
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn offx(&self) -> usize {
        self.offx
    }

    pub fn offy(&self) -> usize {
        self.offy
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.image.get(x + self.offx, y + self.offy)
    }

    pub fn cutout(&self, width: usize, height: usize, offx: usize, offy: usize) -> Self {
        Self {
            image: self.image,
            width,
            height,
            offx: self.offx + offx,
            offy: self.offy + offy,
        }
    }

    pub fn is_blank(&self) -> bool {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn trimm_y(&self, reverse: bool) -> Option<Self> {
        for line in self.lines(reverse) {
            if line.is_blank() {
                continue;
            }
            return Some(Self::new(
                self.image,
                self.width,
                if reverse {
                    line.offy - self.offy
                } else {
                    self.height - (line.offy - self.offy)
                },
                self.offx,
                if reverse { self.offy } else { line.offy },
            ));
        }
        None
    }

    fn trimm_x(&self, reverse: bool) -> Option<Self> {
        for column in self.columns(reverse) {
            if column.is_blank() {
                continue;
            }
            return Some(Self::new(
                self.image,
                if reverse {
                    column.offx - self.offx
                } else {
                    self.width - (column.offx - self.offx)
                },
                self.height,
                if reverse { self.offx } else { column.offx },
                self.offy,
            ));
        }
        None
    }

    pub fn trimm_top(&self) -> Option<Self> {
        self.trimm_y(false)
    }

    pub fn trimm_bottom(&self) -> Option<Self> {
        self.trimm_x(true)
    }

    pub fn trimm_left(&self) -> Option<Self> {
        self.trimm_x(false)
    }

    pub fn trimm_right(&self) -> Option<Self> {
        self.trimm_x(true)
    }

    pub fn till_blank_line(&self) -> Option<Self> {
        for line in self.lines(false) {
            if !line.is_blank() {
                continue;
            }
            return Some(Self::new(
                self.image,
                self.width,
                line.offy - self.offy,
                self.offx,
                self.offy,
            ));
        }
        None
    }

    pub fn columns(&self, reverse: bool) -> CutoutColumnIterator {
        CutoutColumnIterator::new(self, reverse)
    }

    pub fn lines(&self, reverse: bool) -> CutoutLineIterator {
        CutoutLineIterator::new(self, reverse)
    }

    pub fn yparts(&self) -> CutoutYPartIterator {
        CutoutYPartIterator::new(self)
    }

    pub fn pixels(&self, invert: bool, image_scope: bool) -> CutoutPixelIterator {
        CutoutPixelIterator::new(self, invert, image_scope)
    }

    pub fn objects(&self) -> Vec<RectangleCollection> {
        let mut collections: Vec<RectangleCollection> = Vec::new();
        for line in self.lines(false) {
            for rectangle in line.pixels(false, true).to_one_height_rectangles() {
                let touched_collection_ids = collections
                    .iter()
                    .enumerate()
                    .map(|tupel| (tupel.0, tupel.1.bottom_touch(&rectangle)))
                    .filter(|tupel| tupel.1)
                    .map(|tupel| tupel.0)
                    .collect::<Vec<usize>>();
                if touched_collection_ids.len() == 0 {
                    collections.push(RectangleCollection::new(rectangle));
                } else if touched_collection_ids.len() == 1 {
                    collections[touched_collection_ids[0]].add_rectangle(rectangle);
                } else {
                    let mut collection = RectangleCollection::new(rectangle);
                    for id in touched_collection_ids.into_iter().rev() {
                        for rectangle in collections.swap_remove(id).rectangles() {
                            collection.add_rectangle(rectangle)
                        }
                    }
                    collections.push(collection);
                }
            }
        }
        return collections;
    }

    pub fn to_image(&self) -> Image {
        let width = self.width;
        let height = self.height;
        let mut image = Image::new_empty(width, height);
        for y in 0..height {
            for x in 0..width {
                image.set(x, y, self.get(x, y))
            }
        }
        image
    }

    pub fn neighbors(&self, x: usize, y: usize) -> usize {
        let mut result = 0;
        if self.get(x - 1, y) {
            result += 1;
        }
        if self.get(x + 1, y) {
            result += 1;
        }
        if self.get(x, y - 1) {
            result += 1;
        }
        if self.get(x, y + 1) {
            result += 1;
        }
        return result;
    }
}

#[derive(Clone, Debug)]
pub struct CutoutPixelIterator<'a> {
    cutout: &'a Cutout<'a>,
    currx: usize,
    curry: usize,
    invert: bool,
    image_scope: bool,
    xisminusone: bool,
}

impl<'a> CutoutPixelIterator<'a> {
    pub fn new(cutout: &'a Cutout, invert: bool, image_scope: bool) -> Self {
        let currx = 0;
        let curry = 0;
        let xisminusone = true;
        Self {
            cutout,
            currx,
            curry,
            invert,
            image_scope,
            xisminusone,
        }
    }
}

impl<'a> Iterator for CutoutPixelIterator<'a> {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        for y in self.curry..self.cutout.height {
            let xmax = if self.xisminusone {
                self.currx
            } else {
                self.currx + 1
            };
            for x in xmax..(self.cutout.width) {
                self.xisminusone = false;
                if self.cutout.get(x, y) && !self.invert || !self.cutout.get(x, y) && self.invert {
                    self.currx = x;
                    self.curry = y;
                    if self.image_scope {
                        return Some(Pixel::new(
                            self.currx + self.cutout.offx,
                            self.curry + self.cutout.offy,
                        ));
                    } else {
                        return Some(Pixel::new(self.currx, self.curry));
                    }
                }
            }
            self.currx = 0;
            self.xisminusone = true;
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct CutoutColumnIterator<'a> {
    cutout: &'a Cutout<'a>,
    current: usize,
    reverse: bool,
}

impl<'a> CutoutColumnIterator<'a> {
    pub fn new(cutout: &'a Cutout, reverse: bool) -> Self {
        let current = 0;
        Self {
            cutout,
            current,
            reverse,
        }
    }
}

impl<'a> Iterator for CutoutColumnIterator<'a> {
    type Item = Cutout<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.cutout.width {
            return None;
        }

        let image = self.cutout.image;
        let offx = if self.reverse {
            self.cutout.offx + self.cutout.width - 1 - self.current
        } else {
            self.cutout.offx + self.current
        };
        let width = 1;
        let offy = self.cutout.offy;
        let height = self.cutout.height;
        self.current = self.current + 1;
        Some(Cutout::new(image, width, height, offx, offy))
    }
}

#[derive(Clone, Debug)]
pub struct CutoutLineIterator<'a> {
    cutout: &'a Cutout<'a>,
    current: usize,
    reverse: bool,
}

impl<'a> CutoutLineIterator<'a> {
    pub fn new(cutout: &'a Cutout, reverse: bool) -> Self {
        let current = 0;
        Self {
            cutout,
            current,
            reverse,
        }
    }
}

impl<'a> Iterator for CutoutLineIterator<'a> {
    type Item = Cutout<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.cutout.height {
            return None;
        }

        let image = self.cutout.image;
        let offx = self.cutout.offx;
        let width = self.cutout.width;
        let offy = if self.reverse {
            self.cutout.offy + self.cutout.height - 1 - self.current
        } else {
            self.cutout.offy + self.current
        };
        let height = 1;
        self.current = self.current + 1;
        Some(Cutout::new(image, width, height, offx, offy))
    }
}

#[derive(Clone, Debug)]
pub struct CutoutYPartIterator<'a> {
    cutout: Cutout<'a>,
}

impl<'a> CutoutYPartIterator<'a> {
    pub fn new(cutout: &Cutout<'a>) -> Self {
        let cutout = cutout.clone();
        Self { cutout }
    }
}

impl<'a> Iterator for CutoutYPartIterator<'a> {
    type Item = Cutout<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cutout = self.cutout.trimm_top()?;
        let result = self.cutout.till_blank_line()?;
        self.cutout = Cutout::new(
            self.cutout.image,
            self.cutout.width,
            self.cutout.height - result.height,
            0,
            result.offy + result.height + 1,
        );
        Some(result)
    }
}
