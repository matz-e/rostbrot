extern crate clap;
extern crate image;
extern crate rayon;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use cache::{Cache, Configuration};
use clap::{App, Arg};
use rayon::prelude::*;
use std::cmp;
use std::f32;
use std::fs::File;
use std::io;
use std::io::BufWriter;

mod cache;

fn main() -> Result<(), io::Error> {
    let cli = App::new("Rostbrot")
        .version("0.1.0")
        .author("Matthias Wolf <m@sushinara.net>")
        .about("Generate Buddhabrot images")
        .arg(
            Arg::with_name("cache")
                .takes_value(true)
                .long("cache")
                .help("A cache file to use; default: 'cache.yaml'"),
        )
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

    let config_file = File::open(cli.value_of("config").unwrap()).unwrap();
    let config: Configuration = serde_yaml::from_reader(config_file).unwrap();

    let cache_filename = cli.value_of("cache").unwrap_or("cache.yaml");
    let mut cache = Cache::load(&cache_filename, &config);

    if !cache.valid {
        cache.populate();

        let mut f = BufWriter::new(File::create(cache_filename).unwrap());
        match bincode::serialize_into(&mut f, &cache) {
            Ok(r) => r,
            _ => {
                println!("serialization error!");
            }
        };
    }

    let mut imgbuf: image::RgbImage =
        image::ImageBuffer::new(config.dimensions.x as u32, config.dimensions.y as u32);

    let threshold = 5 as f32;
    let luts: Vec<Vec<u8>> = cache
        .layers
        .iter()
        .map(|l| l.data.iter().max().unwrap())
        .map(|m| {
            (0..=*m)
                .into_par_iter()
                .map(|i| {
                    let upper = (*m as f32 - threshold).max(1.0).log2();
                    let value = (i as f32 - threshold).max(1.0).log2();
                    let mapped = (upper / value).sqrt();
                    (mapped * 255.0) as u8
                })
                .collect()
        })
        .collect();

    imgbuf
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            // for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let idx = (x + y * config.dimensions.x as u32) as usize;
            // let v = 0;
            let mut color: [u8; 3] = [0, 0, 0];
            for (i, lut) in luts.iter().enumerate() {
                let v = lut[cache.layers[i].data[idx] as usize];
                for j in 0..color.len() {
                    color[j] = cmp::max(color[j], cmp::min(config.layers[i].color[j], v));
                }
            }
            *pixel = image::Rgb(color);
        });
    imgbuf.save(cli.value_of("filename").unwrap()).unwrap();
    Ok(())
}
