use bim::image::Image;

fn main() {
    let threshold = 0.5;
    let in_path = "./input.png";
    let out_path = "./output.png";
    let clear_border = true;
    let width = 1654;
    let skip_left = true;

    let mut image = Image::from_png(in_path, threshold).unwrap();
    if clear_border {
        image.clear_border();
    }

    let x_min = image.x_min(skip_left);
    let x_max = image.x_max();
    let padding = width - (x_max - x_min);
    let padding_left = padding / 2;
    let padding_right = padding - padding_left;
    image = image.change_border_width(x_min, padding_left, image.width() - x_max, padding_right);

    image.to_png(out_path).unwrap();
}
