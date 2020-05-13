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

pub fn cardioid<T>(c: Complex<T>) -> bool
where
    T: Float,
{
    let one = Complex {
        re: T::from(1.0).unwrap(),
        im: T::from(0.0).unwrap(),
    };
    let mu1 = one + (one - c * T::from(4.0).unwrap()).sqrt();
    let mu2 = one - (one - c * T::from(4.0).unwrap()).sqrt();
    mu1.norm_sqr() < T::from(1.0).unwrap() || mu2.norm_sqr() < T::from(1.0).unwrap()
}

/// Test if the given point is in the first bulb of the Mandelbrot set
///
/// I.e., within a circle of radius 1/4 of (-1, 0).
pub fn first_bulb<T>(c: Complex<T>) -> bool
where
    T: Float,
{
    let center = Complex {
        re: T::from(-1.0).unwrap(),
        im: T::from(0.0).unwrap(),
    };

    (c - center).norm_sqr() < T::from(0.0625).unwrap()
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

    #[test]
    fn cardioid_test() {
        let c = Complex { re: 1.0, im: 0.0 };
        assert!(!cardioid(c));

        let c = Complex { re: 0.0, im: 0.0 };
        assert!(cardioid(c));

        let c = Complex { re: -0.74, im: 0.0 };
        assert!(cardioid(c));
    }

    #[test]
    fn first_bulb_test() {
        let c = Complex { re: 0.0, im: 0.0 };
        assert!(!first_bulb(c));

        let c = Complex { re: -1.24, im: 0.0 };
        assert!(first_bulb(c));

        let c = Complex { re: -0.76, im: 0.0 };
        assert!(first_bulb(c));

        let c = Complex { re: -1.0, im: 0.24 };
        assert!(first_bulb(c));
    }
}
