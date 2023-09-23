pub mod cutout;
pub mod decoder;
pub mod encoder;
pub mod image;
pub mod object;

use image::Image;
use object::Object;

fn main() {
    let mut image_black =
        Image::from_png_filter("input.png", 0.0, 0.0, 0.0, 0.8, 50, true).unwrap();
    {
        let image_yellow =
            Image::from_png_filter("input.png", 254.0, 218.0, 13.0, 0.2, 50, false).unwrap();
        for object in image_yellow.full_cutout().objects(false) {
            for object in image_yellow
                .full_cutout()
                .cutout(
                    object.width(),
                    object.height(),
                    object.xmin(),
                    object.ymin(),
                )
                .objects(true)
                .into_iter()
            {
                image_black.clear_pixels(object)
            }
            image_black.set_pixels(object)
        }
    }
    image_black.to_png("output.png").unwrap();
}
