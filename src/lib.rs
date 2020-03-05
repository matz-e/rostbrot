extern crate num_complex;
extern crate num_traits;

use self::num_traits::cast;
use self::num_traits::Float;
use num_complex::Complex;
use std::iter;
// use std::ops::{Add, Div, Mul, Sub};

struct Binning<T> {
    scale: T,
    min: T,
    num: u16,
}

impl<T> Binning<T>
where
    T: Float, // Add<Output=T> + Div<Output=T> + Mul<Output=T> + Sub<Output=T> + From<u32> + From<f32>
{
    fn bin(&self, n: T) -> Option<u16> {
        match cast((n - self.min) * self.scale) {
            Some(c) => {
                if c >= self.num {
                    None
                } else {
                    Some(c)
                }
            }
            _ => None,
        }
    }

    fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..self.num).map(move |n| {
            self.min + (cast::<u16, T>(n).unwrap() + cast::<f32, T>(0.5).unwrap()) / self.scale
        })
    }
}

pub struct Histogram<'a, T> {
    xaxis: Binning<T>,
    yaxis: Binning<T>,
    bins: Vec<&'a mut [u32]>,
}

impl<'a, T> Histogram<'a, T>
where
    T: Float,
{
    pub fn centers(&self) -> impl Iterator<Item = (T, T)> + '_ {
        self.yaxis
            .iter()
            .flat_map(move |y| self.xaxis.iter().zip(iter::repeat(y)))
    }

    pub fn values(&self, layer: usize) -> impl Iterator<Item = &u32> {
        self.bins[layer].iter()
    }

    pub fn fill(&mut self, i: usize, x: T, y: T) {
        let nx = self.xaxis.bin(x);
        let ny = self.yaxis.bin(y);
        if nx.is_none() || ny.is_none() {
            return;
        }
        let idx = nx.unwrap() as usize + ny.unwrap() as usize * self.xaxis.num as usize;
        self.bins[i][idx] += 1;
    }

    pub fn new(
        xmin: T,
        xmax: T,
        xnum: u16,
        ymin: T,
        ymax: T,
        ynum: u16,
        bins: Vec<&'a mut [u32]>,
    ) -> Histogram<'a, T> {
        let xaxis = Binning {
            scale: T::from(xnum).unwrap() / (xmax - xmin),
            min: xmin,
            num: xnum,
        };
        let yaxis = Binning {
            scale: T::from(ynum).unwrap() / (ymax - ymin),
            min: ymin,
            num: ynum,
        };
        Histogram { xaxis, yaxis, bins }
    }
}

pub struct ComplexSequence<T> {
    z: Complex<T>,
    c: Complex<T>,
    r: T,
}

impl<T> Iterator for ComplexSequence<T>
where
    T: Float,
{
    type Item = Complex<T>;

    fn next(&mut self) -> Option<Complex<T>> {
        self.z = self.z * self.z + self.c;
        if self.z.norm() > self.r {
            return None;
        }
        Some(self.z)
    }
}

pub fn mandelbrot<T>(c: Complex<T>) -> ComplexSequence<T>
where
    T: Float,
{
    let start: T = T::from(0.0).unwrap();
    let radius: T = T::from(2.0).unwrap();
    ComplexSequence {
        z: Complex {
            re: start,
            im: start,
        },
        c,
        r: radius,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binning_iter() {
        let bins = Binning {
            scale: 2.0,
            min: 0.0,
            num: 2,
        };
        let res: Vec<f64> = bins.iter().collect();
        assert_eq!(res, vec![0.25, 0.75]);
    }

    #[test]
    fn binning_fill() {
        let bins = Binning {
            scale: 1.0,
            min: 0.0,
            num: 2,
        };
        let b1 = bins.bin(0.1);
        assert_eq!(b1, Some(0));
        let b2 = bins.bin(5.0);
        assert_eq!(b2, None);
    }

    #[test]
    fn histogram_usage() {
        let mut histo = histogram(0.0, 1.0, 2, 0.0, 1.0, 1);
        let centers: Vec<_> = histo.centers().collect();
        assert_eq!(centers, vec![(0.25, 0.5), (0.75, 0.5)]);
        histo.fill(-2.0, 3.0);
        histo.fill(0.51, 0.1);
        let values: Vec<_> = histo.values().collect();
        assert_eq!(*values[0], 0 as u32);
        assert_eq!(*values[1], 1 as u32);
    }

    #[test]
    fn mandelbrot_seq() {
        let c = Complex { re: 0.0, im: 0.0 };

        let s = mandelbrot(c);
        let res: Vec<_> = s.take(20).collect();
        assert_eq!(res.len(), 20);

        let c = Complex { re: 3.0, im: 0.0 };
        let s = mandelbrot(c);
        let res: Vec<_> = s.take(20).collect();
        assert_eq!(res.len(), 0);

        let c = Complex { re: 1.0, im: 0.0 };
        let s = mandelbrot(c);
        let res: Vec<_> = s.take(20).collect();
        assert_eq!(res, vec![c, Complex { re: 2.0, im: 0.0 }]);
    }
}
