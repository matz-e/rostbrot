extern crate num_complex;

use num_complex::Complex;
use std::iter;

pub struct Binning<T> {
    scale: T,
    min: T,
    num: u32,
}

impl Binning<f64> {
    fn bin(&self, n: f64) -> Option<u32> {
        let cand = ((n - self.min) * self.scale) as i64;
        if cand < 0 || cand >= self.num as i64 {
            return None
        } else {
            return Some(cand as u32);
        }
    }

    fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        (0..self.num).map(move |n|
                          self.min + (n as f64 + 0.5) / self.scale)
    }
}

pub struct Histogram<T> {
    xaxis: Binning<T>,
    yaxis: Binning<T>,
    bins: Vec<u32>,
}

impl Histogram<f64> {
    pub fn centers(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.yaxis.iter()
            .flat_map(move |y| 
                      self.xaxis.iter().zip(iter::repeat(y)))
    }

    pub fn values(&self) -> impl Iterator<Item = &u32> {
        self.bins.iter()
    }

    pub fn fill(&mut self, x: f64, y: f64) {
        let nx = self.xaxis.bin(x);
        let ny = self.yaxis.bin(y);
        if !nx.is_some() || !ny.is_some() {
            return
        }
        let idx = nx.unwrap() + ny.unwrap() * self.xaxis.num;
        self.bins[idx as usize] += 1;
    }
}

pub fn histogram(xmin: f64, xmax: f64, xnum: u32,
                 ymin: f64, ymax: f64, ynum: u32) -> Histogram<f64> {
    let xaxis = Binning { scale: xnum as f64 / (xmax - xmin),
                          min: xmin, num: xnum };
    let yaxis = Binning { scale: ynum as f64 / (ymax - ymin),
                          min: ymin, num: ynum };
    let bins = vec![0; (xnum * ynum) as usize];
    Histogram { xaxis, yaxis, bins }
}

pub struct ComplexSequence<T> {
    z: Complex<T>,
    c: Complex<T>,
}

impl Iterator for ComplexSequence<f64> {
    type Item = Complex<f64>;

    fn next(&mut self) -> Option<Complex<f64>> {
        self.z = self.z * self.z + self.c;
        if self.z.norm() > 2.0 {
            return None
        }
        Some(self.z)
    }
}

pub fn mandelbrot(c: Complex<f64>) -> ComplexSequence<f64> {
    ComplexSequence { z: Complex { re: 0.0, im: 0.0 }, c: c }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binning_iter() {
        let bins = Binning { scale: 2.0, min: 0.0, num: 2 };
        let res: Vec<f64> = bins.iter().collect();
        assert_eq!(res, vec![0.25, 0.75]);
    }

    #[test]
    fn binning_fill() {
        let bins = Binning { scale: 1.0, min: 0.0, num: 2 };
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
