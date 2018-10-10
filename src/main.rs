extern crate image;
extern crate num_complex;
extern crate palette;

use num_complex::Complex;
use palette::{Srgb, LinSrgb, Hsv, Gradient, Pixel};

mod lib;

fn main() {
    let xmin = -2.0;
    let xmax = 1.5;
    let xnum = 2000;
    let ynum = 2000;

    let niter = 500;

    let ymax = (xmax - xmin) * 0.5 * ynum as f64 / xnum as f64;
    let ymin = -ymax;

    let mut histo = lib::histogram(xmin, xmax, xnum,
                                   ymin, ymax, ynum);
    let centers: Vec<_> = histo.centers().collect();
    for (x, y) in centers {
        let c = Complex { re: x, im: y };
        let iters: Vec<_> = lib::mandelbrot(c).take(niter).collect();
        if iters.len() < niter {
            for z in iters {
                histo.fill(z.re, z.im);
            }
        }
    }
    let imax = match histo.values().max() {
        None => 0,
        Some(x) => *x
    };

    let colors: Vec<_> = Gradient::new(vec![
        // Hsv::from(LinSrgb::new(1.0, 0.1, 0.1)),
        // Hsv::from(LinSrgb::new(0.1, 1.0, 0.1)),
        // Hsv::from(LinSrgb::new(0.1, 0.1, 1.0)),
        Hsv::from(LinSrgb::new(0.0, 0.0, 0.0)),
        Hsv::from(LinSrgb::new(1.0, 1.0, 1.0))
    ]).take(imax as usize + 1).collect();

    let data: Vec<[u8; 3]> = histo.values().map(|i|
                                                Srgb::from(colors[*i])
                                                .into_format()
                                                .into_raw()).collect();
    let buffer: &[u8] = &data.concat();
    image::save_buffer("image.png", buffer, xnum as u32, ynum as u32, image::RGB(8)).unwrap()
}
