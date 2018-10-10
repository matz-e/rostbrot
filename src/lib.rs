extern crate num_complex;

use num_complex::Complex;
use std::iter;

pub struct Binning<T> {
    scale: T,
    min: T,
    num: usize,
}

impl Binning<f64> {
    fn bin(&self, n: f64) -> Option<usize> {
        // usize::from((n - self.min) * self.scale).min(self.num).max(0)
        let cand = ((n - self.min) * self.scale) as i64;
        if cand < 0 || cand >= self.num as i64 {
            return None
        } else {
            return Some(cand as usize);
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
    bins: Vec<usize>,
}

impl Histogram<f64> {
    pub fn centers(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.yaxis.iter()
            .flat_map(move |y| 
                      self.xaxis.iter().zip(iter::repeat(y)))
    }

    pub fn values(&self) -> impl Iterator<Item = &usize> {
        self.bins.iter()
    }

    pub fn fill(&mut self, x: f64, y: f64) {
        let nx = self.xaxis.bin(x);
        let ny = self.yaxis.bin(y);
        if !nx.is_some() || !ny.is_some() {
            return
        }
        let idx = nx.unwrap() + ny.unwrap() * self.xaxis.num;
        self.bins[idx] += 1;
    }
}

pub fn histogram(xmin: f64, xmax: f64, xnum: usize,
                 ymin: f64, ymax: f64, ynum: usize) -> Histogram<f64> {
    let xaxis = Binning { scale: xnum as f64 / (xmax - xmin),
                          min: xmin, num: xnum };
    let yaxis = Binning { scale: ynum as f64 / (ymax - ymin),
                          min: ymin, num: ynum };
    let bins = vec![0; xnum * ynum];
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
