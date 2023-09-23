use super::image::Image;
use super::object::Object;
use std::error::Error;
use std::path::Path;

// pub fn clear_border(image: &mut Image) {
//     for object in image.full_cutout().objects(
//         image
//             .full_cutout()
//             .columns(false)
//             .next()
//             .unwrap()
//             .pixels(false, false)
//             .chain(
//                 image
//                     .full_cutout()
//                     .columns(true)
//                     .next()
//                     .unwrap()
//                     .pixels(false, false),
//             )
//             .chain(
//                 image
//                     .full_cutout()
//                     .lines(false)
//                     .next()
//                     .unwrap()
//                     .pixels(false, false),
//             )
//             .chain(
//                 image
//                     .full_cutout()
//                     .lines(true)
//                     .next()
//                     .unwrap()
//                     .pixels(false, false),
//             ),
//     ) {
//         image.clear_pixels(object);
//     }
// }

pub fn add_colored_details(
    image: &mut Image,
    path: impl AsRef<Path>,
    red: f64,
    green: f64,
    blue: f64,
    threshold: f64,
) -> Result<(), Box<dyn Error>> {
    let img_yellow = Image::from_png(path, red, green, blue, threshold)?;
    for part in img_yellow
        .full_cutout()
        .yparts()
        .map(|part| part.trimm_left().unwrap().trimm_right().unwrap())
    {
        image.set_pixels(part.pixels(false, true));
        image.clear_pixels(part.pixels(true, true));
    }
    Ok(())
}

pub fn remove_salt_and_pepper(image: &mut Image, height_threshold: usize) -> Image {
    let mut result = Image::new_empty(image.width(), image.height());
    for part in image
        .full_cutout()
        .yparts()
        .filter(|part| part.height() > height_threshold)
    {
        result.set_pixels(part.pixels(false, true));
    }
    result
}

pub fn horizontal_padding(image: &Image, width: usize) -> Result<Image, Box<dyn Error>> {
    let old_width = image.width();
    let cutout = image.full_cutout().trimm_left().unwrap();
    let trimmed_width = cutout.width();
    let mut result = Image::new_empty(width, image.height());
    let pixels_to_move = (width - old_width) / 2 - (old_width - trimmed_width);
    result.set_pixels(
        cutout
            .pixels(false, true)
            .map(|pixel| pixel.addx(pixels_to_move)),
    );
    Ok(result)
}
