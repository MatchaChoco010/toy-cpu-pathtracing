use image::{Rgb, RgbImage};
use rayon::prelude::*;

fn main() {
    let width = 800;
    let height = 600;

    let mut img = RgbImage::new(width, height);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            pixel[0] = (255.0 * (*x as f64 / width as f64)) as u8;
            pixel[1] = (255.0 * (*y as f64 / height as f64)) as u8;
            pixel[2] = 128;
        });
    img.save("output.png").unwrap();
}
