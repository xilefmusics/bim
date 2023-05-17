use bim::image::Image;

fn main() {
    let mut image = Image::from_png("./input.png", 0.5).unwrap();
    image.clear_border();
    image.to_png("./output.png").unwrap();
}
