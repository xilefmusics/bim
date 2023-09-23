pub mod cutout;
pub mod decoder;
pub mod encoder;
pub mod image;
pub mod object;
pub mod opt;

use image::Image;
use object::Object;

fn main() {
    let image_black = Image::from_png("input.png", 0.0, 0.0, 0.0, 0.8).unwrap();
    let mut image_out = Image::new_empty(image_black.width(), image_black.height());
    for object in image_black
        .full_cutout()
        .objects()
        .into_iter()
        .filter(|object| {
            object.size() > 50 && !object.touches_border(image_black.width(), image_black.height())
        })
    {
        image_out.set_pixels(object)
    }
    image_out.to_png("output.png").unwrap();
}
