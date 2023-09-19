pub mod cutout;
pub mod decoder;
pub mod encoder;
pub mod image;

use cutout::Cutout;
use image::Image;

fn main() {
    let image = {
        let mut img_black = Image::from_png("input.png", 0.0, 0.0, 0.0, 0.8).unwrap();
        let img_yellow = Image::from_png("input.png", 254.0, 218.0, 13.0, 0.2).unwrap();
        let cutout = img_yellow.full_cutout();
        for part in cutout.yparts() {
            let part = part.trimm_left().unwrap().trimm_right().unwrap();
            let mut image = part.to_image();
            image.fill_border();
            img_black.overwrite(&image, part.offx(), part.offy()); // TODO use pixel iterator with
                                                                   // transformation
        }
        img_black
    };

    image.to_png("output.png").unwrap();

    let cutout = image.full_cutout();
    let parts = cutout.yparts().collect::<Vec<Cutout>>();
    for (idx, part) in parts.iter().enumerate() {
        let image = part.to_image();
        image.to_png(format!("0_parts/{:03}.png", idx)).unwrap();
    }
}
