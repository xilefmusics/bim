use bim::image::Image;

fn main() {
    let image = Image::from_png("./input.png", 0.5).unwrap();
    image.to_png("./output.png").unwrap();
}
