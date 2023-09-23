pub mod cutout;
pub mod decoder;
pub mod encoder;
pub mod image;
pub mod object;
pub mod opt;

use image::Image;

fn main() {
    let image_black = Image::from_png_filter("input.png", 0.0, 0.0, 0.0, 0.8, 50).unwrap();
    let image_yellow = Image::from_png_filter("input.png", 254.0, 218.0, 13.0, 0.2, 50).unwrap();
    image_yellow.to_png("output.png").unwrap();
}
