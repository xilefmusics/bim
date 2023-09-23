pub mod cutout;
pub mod decoder;
pub mod encoder;
pub mod image;
pub mod object;

use clap::Parser;
use image::Image;
use object::Object;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The input path of the png
    pub input_path: String,
    /// The output path of the png
    pub output_path: String,
    /// The threshold that defines when a pixel gets set to 0 or 1 (its a value between 0 and 1)
    #[arg(short, long, default_value_t = 0.5)] // 0.8
    pub threshold: f64,
    /// The threshold of the pixel size of objects that are filter out as salt and pepper
    #[arg(short, long, default_value_t = 0)] // 50
    pub obj_threshold: usize,
    /// The threshold to read in a specific coller
    #[arg(short, long, default_value_t = 0.0)] // 0.2
    pub color_threshold: f64,
    /// The red channel of the extra coller to read in
    #[arg(short, long, default_value_t = 0)] // 254
    pub red: u8,
    /// The green channel of the extra collor to read in
    #[arg(short, long, default_value_t = 0)] // 218
    pub green: u8,
    /// The blue channel of the extra collor to read in
    #[arg(short, long, default_value_t = 0)] // 13
    pub blue: u8,
    /// The width to which the image should be padded
    #[arg(short, long, default_value_t = 0)] // 2480
    pub width: usize,
}

fn main() {
    let args = Args::parse();

    let mut image_black = Image::from_png_filter(
        args.input_path.clone(),
        0.0,
        0.0,
        0.0,
        args.threshold,
        args.obj_threshold,
        true,
    )
    .unwrap();
    if args.color_threshold > 0.0 {
        let image_yellow = Image::from_png_filter(
            args.input_path,
            args.red as f64,
            args.green as f64,
            args.blue as f64,
            args.color_threshold,
            args.obj_threshold,
            false,
        )
        .unwrap();
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

    if args.width > 0 {
        image_black = image_black.horizontal_padding(args.width).unwrap()
    }
    image_black.to_png(args.output_path).unwrap();
}
