extern crate bincode;
extern crate num_complex;
extern crate pbr;
extern crate rayon;

#[cfg(test)]
extern crate tempfile;

#[path = "histogram.rs"]
mod histogram;
#[path = "mandelbrot.rs"]
mod mandelbrot;

use self::histogram::Histogram;
use self::mandelbrot::mandelbrot;
use self::num_complex::Complex;
use self::pbr::ProgressBar;
use self::rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize)]
pub struct Layer {
    iterations: usize,
    pub color: [u8; 3],
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LayerData {
    iterations: usize,
    pub data: Vec<u32>,
}

impl PartialEq<Layer> for LayerData {
    fn eq(&self, other: &Layer) -> bool {
        self.iterations == other.iterations
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Area {
    x: [f32; 2],
    y: [f32; 2],
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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

impl Configuration {
    pub fn load(filename: &str) -> Result<Configuration, Box<dyn Error>> {
        let file = File::open(filename)?;
        let config = serde_yaml::from_reader(file)?;
        Ok(config)
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

    pub fn dump(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let mut f = BufWriter::new(File::create(filename)?);
        bincode::serialize_into(&mut f, &self)?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use self::tempfile::{tempdir,TempDir};
    use std::io::Write;
    use super::*;

    #[test]
    fn layer_equality() {
        let ld = LayerData { iterations: 10, data: vec![] };
        let l = Layer { iterations: 10, color: [0, 0, 0] };
        assert_eq!(ld, l);
        let l2 = Layer { iterations: 1, color: [0, 0, 0] };
        assert_ne!(ld, l2);
    }

    #[test]
    fn dimensionality() {
        let d = Dimensions { x: 5, y: 4 };
        assert_eq!(d.size(), 20);
    }

    fn dump_config(dir: &TempDir) -> Configuration {
        let path = dir.path().join("config.yaml");
        let filename = path.to_str().unwrap();
        {
            let mut file = File::create(filename).unwrap();
            writeln!(file, r#"
                dimensions:
                  x: 10
                  y: 5

                area:
                  x: [-2, 2]
                  y: [-1, 1]

                layers:
                  - iterations: 10
                    color: [100, 100, 100]
                  - iterations: 1
                    color: [10, 10, 10]
            "#).unwrap();
        }
        Configuration::load(filename).unwrap()
    }

    #[test]
    fn load_config() {
        let dir = tempdir().unwrap();
        let config = dump_config(&dir);
        assert_eq!(config.dimensions.x, 10);
        assert_eq!(config.area.x[1], 2.0);
        assert_eq!(config.layers[0].iterations, 10);
    }

    #[test]
    fn restore_cache() {
        let dir = tempdir().unwrap();
        let config = dump_config(&dir);
        let cache = Cache::new(&config);
        assert_eq!(cache.valid, false);
        assert_eq!(cache.layers.len(), 2);
        {
            let path = dir.path().join("cache.bin");
            let filename = path.to_str().unwrap();
            cache.dump(filename).unwrap();
            let restored = Cache::load(filename, &config);
            assert_eq!(cache, restored);
        }
    }

    #[test]
    fn restore_modified_cache() {
        let dir = tempdir().unwrap();
        let mut config = dump_config(&dir);
        let mut cache = Cache::new(&config);
        {
            let path = dir.path().join("cache.bin");
            let filename = path.to_str().unwrap();
            cache.valid = true;
            cache.dump(filename).unwrap();
            config.layers[0].iterations = 100;
            let restored = Cache::load(filename, &config);
            assert_eq!(restored.valid, false);
            assert_ne!(restored, cache);
        }
    }
}
