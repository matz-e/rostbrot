extern crate image;
extern crate num_complex;
extern crate palette;

use num_complex::Complex;
use palette::{Srgb, LinSrgb, Hsv, Gradient, Pixel};

mod lib;

fn main() {
    let xmin = -2.3;
    let xmax = 0.7;
    let xnum = 4000;
    let ynum = 3000;

    let niter = 100;

    let ymax = (xmax - xmin) * 0.5 * ynum as f64 / xnum as f64;
    let ymin = -ymax;

    let coords = lib::span(ymin, ymax, ynum).flat_map(
        |y| lib::span(xmin, xmax, xnum).zip(std::iter::repeat(y))
                                       .map(|(x, y)| Complex { re: x, im: y }));
    let niters: Vec<_> = coords.map(|c| { let res: Vec<_> = lib::mandelbrot(c).take(niter).collect(); res.len() }).collect();
    let imax = match niters.iter().max() {
        None => 0,
        Some(x) => *x
    };

    let colors: Vec<_> = Gradient::new(vec![
        Hsv::from(LinSrgb::new(1.0, 0.1, 0.1)),
        Hsv::from(LinSrgb::new(0.1, 1.0, 0.1)),
        Hsv::from(LinSrgb::new(0.1, 0.1, 1.0)),
        Hsv::from(LinSrgb::new(0.0, 0.0, 0.0))
    ]).take(imax as usize + 1).collect();

    let data: Vec<[u8; 3]> = niters.iter().map(|i|
                                               Srgb::from(colors[*i])
                                               .into_format()
                                               .into_raw()).collect();
    let buffer: &[u8] = &data.concat();
    image::save_buffer("image.png", buffer, xnum, ynum, image::RGB(8)).unwrap()
}
