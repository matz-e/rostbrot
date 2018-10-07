extern crate num_complex;

use num_complex::Complex;

pub struct LinearSequence {
    start: f64,
    end: f64,
    step: f64,
    counter: u32,
}

impl Iterator for LinearSequence {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        let val = (self.counter as f64 + 0.5) * self.step + self.start;
        self.counter = self.counter + 1;
        if self.end <= val {
            return None;
        }
        return Some(val);
    }
}

pub fn span(min: f64, max: f64, num: u32) -> LinearSequence {
    let step = (max - min) / num as f64;
    LinearSequence { start: min, end: max, step, counter: 0 }
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
    fn linear_seq() {
        let res: Vec<_> = span(0.0, 1.0, 2).collect();
        assert_eq!(res, vec![0.25, 0.75]);
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
