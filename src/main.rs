extern crate image;
extern crate num_complex;
extern crate palette;

use num_complex::Complex;
use palette::{Srgb, LinSrgb, Hsv, Gradient, Pixel};

fn iterate(z: Complex<f64>, c: Complex<f64>, nmax: usize) -> usize {
    let mut y = z;
    for i in 0..nmax {
        y = y * y + c;
        if y.norm() > 2.0 {
            return i;
        }
    }
    return nmax;
}

struct Expanse {
    min: f64,
    max: f64,
    num: i32
}

fn main() {
    let xmin = -2.3;
    let xmax = 0.7;
    let xnum = 4000;
    let ynum = 3000;

    let niter = 100;

    let ymax = (xmax - xmin) * 0.5 * ynum as f64 / xnum as f64;
    let ymin = -ymax;

    let dx = (xmax - xmin) / (xnum as f64);
    let dy = (ymax - ymin) / (ynum as f64);

    let coords = (0..ynum).flat_map(
        |y| (0..xnum).zip(std::iter::repeat(y))
                   .map(|(x, y)|
                        Complex { re: xmin + dx * (x as f64 + 0.5),
                                  im: ymin + dy * (y as f64 + 0.5)}));
    let zero = Complex { re: 0.0, im: 0.0 };
    let niters: Vec<_> = coords.map(|c| iterate(zero, c, niter)).collect();
    let imax = match niters.iter().max() {
        None => 0,
        Some(x) => *x
    };

    let colors: Vec<_> = Gradient::new(vec![
        Hsv::from(LinSrgb::new(1.0, 0.1, 0.1)),
        Hsv::from(LinSrgb::new(0.1, 1.0, 1.0)),
        Hsv::from(LinSrgb::new(0.0, 0.0, 0.0))
    ]).take(imax as usize + 1).collect();

    let data: Vec<[u8; 3]> = niters.iter().map(|i|
                                               Srgb::from(colors[*i])
                                               .into_format()
                                               .into_raw()).collect();
    let buffer: &[u8] = &data.concat();
    image::save_buffer("image.png", buffer, xnum, ynum, image::RGB(8)).unwrap()
}
