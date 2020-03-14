extern crate num_complex;
extern crate num_traits;

use self::num_complex::Complex;
use self::num_traits::Float;

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
        if self.z.norm_sqr() > self.r * self.r {
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
