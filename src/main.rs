extern crate bincode;
extern crate clap;
extern crate image;
extern crate num_complex;
extern crate pbr;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use clap::{App, Arg};
use lib::{mandelbrot, Histogram};
use num_complex::Complex;
use pbr::ProgressBar;
use std::cmp;
use std::fs::File;
use std::io;
use std::io::BufWriter;

mod lib;

#[derive(Deserialize)]
struct Layer {
    iterations: usize,
}

#[derive(Deserialize, Serialize)]
struct LayerData {
    iterations: usize,
    data: Vec<u32>,
}

impl PartialEq<Layer> for LayerData {
    fn eq(&self, other: &Layer) -> bool {
        self.iterations == other.iterations
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
struct Area {
    x: [f32; 2],
    y: [f32; 2],
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
struct Dimensions {
    x: u16,
    y: u16,
}

impl Dimensions {
    fn size(self) -> usize {
        self.x as usize * self.y as usize
    }
}

#[derive(Deserialize)]
struct Configuration {
    area: Area,
    dimensions: Dimensions,
    layers: Vec<Layer>,
}

#[derive(Deserialize, Serialize)]
struct Cache {
    area: Area,
    dimensions: Dimensions,
    layers: Vec<LayerData>,
    valid: bool,
}

impl PartialEq<Configuration> for Cache {
    fn eq(&self, other: &Configuration) -> bool {
        if self.area != other.area
            || self.dimensions != other.dimensions
            || self.layers.len() != other.layers.len()
        {
            return false;
        }
        for (&ref a, &ref b) in self.layers.iter().zip(other.layers.iter()) {
            if a != b {
                return false;
            }
        }
        true
    }
}

impl Cache {
    fn new(c: &Configuration) -> Cache {
        Cache {
            area: c.area,
            dimensions: c.dimensions,
            layers: c
                .layers
                .iter()
                .map(|l| LayerData {
                    iterations: l.iterations,
                    data: vec![0; c.dimensions.size()],
                })
                .collect(),
            valid: false,
        }
    }

    fn load(filename: &str, config: &Configuration) -> Cache {
        let file = match File::open(filename) {
            Ok(f) => f,
            _ => return Cache::new(config),
        };
        match bincode::deserialize_from(file) {
            Ok(c) => {
                if c == *config {
                    c
                } else {
                    Cache::new(config)
                }
            }
            _ => Cache::new(config),
        }
    }

    fn populate(&mut self) {
        let max_iter = match self.layers.iter().map(|l| l.iterations).max() {
            Some(n) => n,
            None => 0,
        };

        let iterations: Vec<_> = self
            .layers
            .iter()
            .map(|l| l.iterations)
            .enumerate()
            .collect();

        let mut data: Vec<&mut [u32]> = vec![];
        for l in self.layers.iter_mut() {
            data.push(&mut l.data[..]);
        }

        let mut histo = Histogram::new(
            self.area.x[0],
            self.area.x[1],
            self.dimensions.x,
            self.area.y[0],
            self.area.y[1],
            self.dimensions.y,
            data,
        );

        let mut pbar = ProgressBar::new(self.dimensions.size() as u64);
        pbar.show_counter = false;
        pbar.show_percent = false;
        pbar.show_speed = false;
        let msg = format!("{} iterations per pixel ", max_iter);
        pbar.message(&msg);

        let centers: Vec<_> = histo.centers().collect();
        for (x, y) in centers {
            let c = Complex { re: x, im: y };
            let nums: Vec<_> = mandelbrot(c).take(max_iter).collect();
            for (layer, maximum) in iterations.iter() {
                if nums.len() < *maximum {
                    for z in nums.iter() {
                        histo.fill(*layer, z.re, z.im);
                    }
                }
            }
            pbar.inc();
        }

        pbar.finish();

        // FIXME needs to be cross checked!
        self.valid = true;
    }
}

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
    let threshold: u32 = 5;
    let maxvalue = cache.layers[0].data.iter().max().unwrap() - threshold;
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let idx = (x + y * config.dimensions.x as u32) as usize;
        let v = (cmp::max(
            0,
            cmp::max(cache.layers[0].data[idx], threshold) - threshold,
        ) * 255
            / maxvalue) as u8;
        *pixel = image::Rgb([v, v, v]);
    }
    imgbuf.save(cli.value_of("filename").unwrap()).unwrap();
    Ok(())
}
