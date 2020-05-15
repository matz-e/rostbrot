extern crate image;
extern crate bincode;
#[macro_use]
extern crate log;
extern crate num_complex;
extern crate num_traits;
extern crate pbr;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

#[cfg(test)]
extern crate tempfile;

pub mod cache;
pub mod color;
pub mod mandelbrot;
