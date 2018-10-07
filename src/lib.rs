extern crate num_complex;

use num_complex::Complex;

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
