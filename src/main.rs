use bim::image::Image;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The input path of the png
    pub input_path: String,
    /// The output path of the png
    pub output_path: String,
    /// The threshold that defines when a pixel gets set to 0 or 1 (its a value between 0 and 1)
    #[arg(short, long, default_value_t = 0.5)]
    pub threshold: f64,
    /// A secondary threshold with whitch the image gets read a second time and only touching
    /// pixels are set
    #[arg(long, default_value_t = 0.0)]
    pub threshold2: f64,
    /// If set each object that touches the border gets removed
    #[arg(short, long, default_value_t = false)]
    pub clear_border: bool,
    /// Skips artifacts on the right border when chaning the padding of the width
    #[arg(long, default_value_t = false)]
    pub skip_left: bool,
    /// Skips artifacts on the left border when changing the padding of the width
    #[arg(long, default_value_t = false)]
    pub skip_right: bool,
    /// The width the image gets padded to
    #[arg(short, long, default_value_t = 0)]
    pub width: usize,
}

fn main() {
    let args = Args::parse();

    let mut image = Image::from_png(&args.input_path, args.threshold).unwrap();
    if args.clear_border {
        image.clear_border();
    }
    if args.threshold2 > 0.0 {
        let image2 = Image::from_png(&args.input_path, args.threshold2).unwrap();
        image.merge_grow(&image2);
        if args.clear_border {
            image.clear_border();
        }
    }

    let x_min = image.x_min(args.skip_left);
    let x_max = image.x_max(args.skip_right);
    let (padding_left, padding_right) = if args.width > 0 {
        let padding = args.width - (x_max - x_min);
        let padding_left = padding / 2;
        let padding_right = padding - padding_left;
        (padding_left, padding_right)
    } else {
        (x_min, image.width() - x_max)
    };
    image = image.change_border_width(x_min, padding_left, image.width() - x_max, padding_right);

    image.to_png(&args.output_path).unwrap();
}
