extern crate bincode;
extern crate num_complex;
extern crate pbr;
extern crate rayon;

#[path = "histogram.rs"]
mod histogram;
#[path = "mandelbrot.rs"]
mod mandelbrot;

use self::histogram::Histogram;
use self::mandelbrot::mandelbrot;
use self::num_complex::Complex;
use self::pbr::ProgressBar;
use self::rayon::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct Layer {
    iterations: usize,
    pub color: [u8; 3],
}

#[derive(Deserialize, Serialize)]
pub struct LayerData {
    iterations: usize,
    pub data: Vec<u32>,
}

impl PartialEq<Layer> for LayerData {
    fn eq(&self, other: &Layer) -> bool {
        self.iterations == other.iterations
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct Area {
    x: [f32; 2],
    y: [f32; 2],
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct Dimensions {
    pub x: u16,
    pub y: u16,
}

impl Dimensions {
    pub fn size(self) -> usize {
        self.x as usize * self.y as usize
    }
}

#[derive(Deserialize)]
pub struct Configuration {
    area: Area,
    pub dimensions: Dimensions,
    pub layers: Vec<Layer>,
}

#[derive(Deserialize, Serialize)]
pub struct Cache {
    area: Area,
    dimensions: Dimensions,
    pub layers: Vec<LayerData>,
    pub valid: bool,
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
    pub fn new(c: &Configuration) -> Cache {
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

    pub fn load(filename: &str, config: &Configuration) -> Cache {
        let file = match File::open(filename) {
            Ok(f) => f,
            _ => return Cache::new(config),
        };
        let buf_reader = BufReader::new(file);
        match bincode::deserialize_from(buf_reader) {
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

    pub fn populate(&mut self) {
        let max_iter = match self.layers.iter().map(|l| l.iterations).max() {
            Some(n) => n,
            None => 0,
        };

        let iterations: Vec<_> = self.layers.iter().map(|l| l.iterations).collect();

        let area = self.area;
        let dimensions = self.dimensions;

        let histos: Vec<_> = self
            .layers
            .iter_mut()
            .map(|layer| {
                Arc::new(Mutex::new(Histogram::new(
                    area.x[0],
                    area.x[1],
                    dimensions.x,
                    area.y[0],
                    area.y[1],
                    dimensions.y,
                    &mut layer.data[..],
                )))
            })
            .collect();

        let mut pbar = ProgressBar::new(self.dimensions.size() as u64);
        pbar.show_counter = false;
        pbar.show_percent = false;
        pbar.show_speed = false;
        let msg = format!("{} iterations per pixel ", max_iter);
        pbar.message(&msg);

        let centers: Vec<_> = histos[0].lock().unwrap().centers().collect();
        let pbarp = Arc::new(Mutex::new(pbar));
        let batchsize = 1000;

        centers.par_chunks(batchsize).for_each(|chunk| {
            for &(x, y) in chunk {
                let c = Complex { re: x, im: y };
                let nums: Vec<_> = mandelbrot(c).take(max_iter).collect();
                for (mutex, maximum) in histos.iter().zip(iterations.iter()) {
                    if nums.len() < *maximum {
                        let mut hist = mutex.lock().unwrap();
                        for z in nums.iter() {
                            hist.fill(z.re, z.im);
                        }
                    }
                }
            }
            pbarp.lock().unwrap().add(batchsize as u64);
        });

        pbarp.lock().unwrap().finish();

        self.valid = true;
    }
}
