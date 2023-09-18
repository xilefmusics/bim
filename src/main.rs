pub mod cutout;
pub mod decoder;
pub mod encoder;
pub mod image;

use cutout::ReferencedCutout;
use image::Image;

fn main() {
    let img_black = Image::from_png("input.png", 0.0, 0.0, 0.0, 0.8).unwrap();
    img_black.to_png("0_output_black.png").unwrap();

    let img_yellow = Image::from_png("input.png", 254.0, 218.0, 13.0, 0.2).unwrap();
    img_yellow.to_png("0_output_yellow.png").unwrap();

    let img_and = img_black.and(&img_yellow).unwrap();
    img_and.to_png("0_output_and.png").unwrap();

    let cutout = img_yellow.full_cutout();
    let parts = cutout.yparts().collect::<Vec<ReferencedCutout>>();
    for (idx, part) in parts.iter().enumerate() {
        part.trimm_left()
            .unwrap()
            .trimm_right()
            .unwrap()
            .to_image()
            .to_png(format!("0_part{}.png", idx))
            .unwrap();
    }
    println!("{:?}", parts);
}
