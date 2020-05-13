extern crate image;
extern crate rayon;

extern crate clap;
extern crate num_complex;
extern crate rostbrot;

use rostbrot::cache::{Cache, Configuration};
use rostbrot::mandelbrot::cardioid;

use clap::{App, Arg};
use num_complex::Complex;
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = App::new("Rostbrot")
        .version("0.1.0")
        .author("Matthias Wolf <m@sushinara.net>")
        .about("Generate the cardiod of Mandelbrot fractals")
        .arg(
            Arg::with_name("config")
                .takes_value(true)
                .required(true)
                .index(1)
                .help("A yaml configuration file"),
        )
        .arg(
            Arg::with_name("filename")
                .takes_value(true)
                .required(true)
                .index(2)
                .help("The output filename"),
        )
        .get_matches();

    let config_filename = cli.value_of("config").unwrap();
    let config_filestub = Path::new(config_filename).file_stem().unwrap();
    let config = Configuration::load(config_filename).unwrap();

    let cache_filename_default = format!("{}.cache", config_filestub.to_str().unwrap());
    let cache_filename = cli.value_of("self").unwrap_or(&cache_filename_default);
    let mut cache = Cache::load(&cache_filename, &config);

    if !cache.valid {
        cache.populate();
        cache.dump(cache_filename).unwrap();
    }

    let filename = cli.value_of("filename").unwrap();
    let mut imgbuf: image::RgbImage =
        image::ImageBuffer::new(config.dimensions.x as u32, config.dimensions.y as u32);

    imgbuf
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            let re = (x as f32 + 0.5) / config.dimensions.x as f32
                * (config.area.x[1] - config.area.x[0])
                + config.area.x[0];
            let im = (y as f32 + 0.5) / config.dimensions.y as f32
                * (config.area.y[1] - config.area.y[0])
                + config.area.y[0];
            let c = Complex { re, im };
            if cardioid(c) {
                *pixel = image::Rgb([0, 0, 0]);
            } else {
                *pixel = image::Rgb([200, 200, 200]);
            }
        });
    imgbuf.save(filename)?;
    Ok(())
}
