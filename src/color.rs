extern crate image;
extern crate rayon;

#[path = "cache.rs"]
mod cache;

use self::rayon::prelude::*;
use std::error::Error;
use std::cmp;
use std::f32;

use cache::{Cache, Configuration};

pub fn colorize(cache: &Cache, config: &Configuration, filename: &str) -> Result<(), Box<dyn Error>> {
    let mut imgbuf: image::RgbImage =
        image::ImageBuffer::new(config.dimensions.x as u32, config.dimensions.y as u32);

    let threshold = 5.0f32;
    let luts: Vec<Vec<u8>> = cache
        .layers
        .iter()
        .map(|l| {
            let m = l.data.iter().max().unwrap();
            (0..=*m)
                .into_par_iter()
                .map(|i| {
                    let upper = (*m as f32 - threshold).max(1.0).log2();
                    let value = (i as f32 - threshold).max(1.0).log2();
                    let mapped = (value / upper).sqrt();
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
                for (j, col) in color.iter_mut().enumerate() {
                    *col = cmp::max(*col, cmp::min(config.layers[i].color[j], v));
                }
            }
            *pixel = image::Rgb(color);
        });
    imgbuf.save(filename)?;
    Ok(())
}
