extern crate bincode;
extern crate clap;
extern crate image;
extern crate num_complex;
extern crate palette;
extern crate pbr;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use bincode::{deserialize_from, serialize_into};
use clap::{App, Arg};
use num_complex::Complex;
use palette::{Gradient, Hsv, LinSrgb, Pixel, Srgb};
use pbr::ProgressBar;
use std::fs::File;
use std::io;
use std::io::BufWriter;

mod lib;

#[derive(Deserialize)]
struct Layer {
    iterations: usize,
    colors: Vec<[f32; 3]>,
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
    fn size(&self) -> usize {
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
}

fn color_cache<'a>(
    cache: &'a Cache,
    config: &'a Configuration,
) -> impl Iterator<Item = impl Iterator<Item = LinSrgb> + 'a> + 'a {
    cache.layers.iter().zip(config.layers.iter()).map(|(d, c)| {
        let imax = match d.data.iter().max() {
            Some(&n) => n,
            None => 0,
        };

        let gradient: Gradient<Hsv> = Gradient::new(
            c.colors
                .iter()
                .map(|[r, g, b]| Hsv::from(Srgb::new(*r, *g, *b))),
        );

        let colors: Vec<_> = gradient.take(imax as usize + 1).collect();
        d.data
            .iter()
            .map(move |&i| LinSrgb::from(colors[i as usize]))
    })
}

fn populate_cache(cache: &mut Cache) {
    let max_iter = match cache.layers.iter().map(|l| l.iterations).max() {
        Some(n) => n,
        None => 0,
    };

    let iterations: Vec<_> = cache
        .layers
        .iter()
        .map(|l| l.iterations)
        .enumerate()
        .collect();

    let mut data: Vec<&mut [u32]> = vec![];
    for l in cache.layers.iter_mut() {
        data.push(&mut l.data[..]);
    }

    let mut histo = lib::Histogram::new(
        cache.area.x[0],
        cache.area.x[1],
        cache.dimensions.x,
        cache.area.y[0],
        cache.area.y[1],
        cache.dimensions.y,
        data,
    );

    let mut bar = ProgressBar::new(cache.dimensions.size() as u64);
    bar.show_counter = false;
    bar.show_percent = false;
    bar.show_speed = false;
    let msg = format!("{} iterations per pixel ", max_iter);
    bar.message(&msg);

    let centers: Vec<_> = histo.centers().collect();
    for (x, y) in centers {
        let c = Complex { re: x, im: y };
        let nums: Vec<_> = lib::mandelbrot(c).take(max_iter).collect();
        for (layer, maximum) in iterations.iter() {
            if nums.len() < *maximum {
                for z in nums.iter() {
                    histo.fill(*layer, z.re, z.im);
                }
            }
        }
        bar.inc();
    }

    bar.finish();

    // FIXME needs to be cross checked!
    cache.valid = true;
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
        populate_cache(&mut cache);

        let mut f = BufWriter::new(File::create(cache_filename).unwrap());
        match serialize_into(&mut f, &cache) {
            Ok(r) => r,
            _ => {
                println!("serialization error!");
            }
        };
    }

    let data: Vec<LinSrgb> = vec![LinSrgb::new(0.0, 0.0, 0.0); cache.dimensions.size()];

    // for layer in config.layers {
    //     let mapped: Vec<_> = histo.values().map(|i| (((*i as f64 + 1.0).ln() + 1.0).ln().powf(0.4) * 100.0) as usize).collect();
    //     let imax = match mapped.iter().max() {
    //         None => 0,
    //         Some(x) => *x
    //     };

    // }

    let temp: Vec<[u8; 3]> = data.iter().map(|c| c.into_format().into_raw()).collect();
    let buffer: &[u8] = &temp.concat();
    image::save_buffer(
        cli.value_of("filename").unwrap(),
        buffer,
        config.dimensions.x as u32,
        config.dimensions.y as u32,
        image::RGB(8),
    )
    .unwrap();
    Ok(())
}
