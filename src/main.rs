extern crate clap;
extern crate image;
extern crate num_complex;
extern crate palette;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use clap::{App, Arg};
use num_complex::Complex;
use palette::{Blend, Gradient, Hsv, LinSrgba, Srgba, Pixel};
use std::fs::File;

mod lib;

#[derive(Deserialize)]
struct Layer {
    iterations: usize,
    colors: Vec<[f32; 4]>
}

#[derive(Deserialize)]
struct Dimensions {
    x: u32,
    y: u32
}

#[derive(Deserialize)]
struct Area {
    x: [f64; 2],
    y: [f64; 2]
}

#[derive(Deserialize)]
struct Configuration {
    area: Area,
    dimensions: Dimensions,
    layers: Vec<Layer>,
}

fn main() {
    let matches = App::new("Rostbrot")
        .version("0.1.0")
        .author("Matthias Wolf <m@sushinara.net>")
        .about("Generate Buddhabrot images")
        .arg(Arg::with_name("config")
                 .takes_value(true)
                 .required(true)
                 .index(1)
                 .help("A yaml configuration file"))
        .arg(Arg::with_name("filename")
                 .takes_value(true)
                 .required(true)
                 .index(2)
                 .help("The output filename"))
        .get_matches();

    let config_file = File::open(matches.value_of("config").unwrap()).unwrap();
    let config: Configuration = serde_yaml::from_reader(config_file).unwrap();

    let ntotal = (config.dimensions.x * config.dimensions.y) as usize;
    let mut data: Vec<LinSrgba> = vec![LinSrgba::new(0.0, 0.0, 0.0, 0.0); ntotal];

    for layer in config.layers {
        println!("Performing {} iterations per pixel", layer.iterations);
        let mut histo = lib::histogram(config.area.x[0],
                                       config.area.x[1],
                                       config.dimensions.x,
                                       config.area.y[0],
                                       config.area.y[1],
                                       config.dimensions.y);

        let centers: Vec<_> = histo.centers().collect();
        for (x, y) in centers {
            let c = Complex { re: x, im: y };
            let iters: Vec<_> = lib::mandelbrot(c).take(layer.iterations)
                                                  .collect();
            if iters.len() < layer.iterations {
                for z in iters {
                    histo.fill(z.re, z.im);
                }
            }
        }
        let imax = match histo.values().max() {
            None => 0,
            Some(x) => *x
        };

        let gradient: Gradient<Hsv> = Gradient::new(
            layer.colors.iter()
                        .map(|[r, g, b, a]|
                             Hsv::from(Srgba::new(*r, *g, *b, *a)))
        );
        let colors: Vec<_> = gradient.take(imax as usize + 1).collect();
        for (n, i) in histo.values().enumerate() {
            let color = LinSrgba::from(colors[*i as usize]);
            data[n] = data[n].overlay(color);
        }
    }

    let temp: Vec<[u8; 4]> = data.iter()
                                 .map(|c| c.into_format()
                                           .into_raw())
                                 .collect();
    let buffer: &[u8] = &temp.concat();
    image::save_buffer(matches.value_of("filename").unwrap(),
                       buffer,
                       config.dimensions.x,
                       config.dimensions.y,
                       image::RGBA(8)).unwrap()
}
