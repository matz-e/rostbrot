extern crate num_traits;

use self::num_traits::cast;
use self::num_traits::Float;
use std::iter;

struct Binning<T> {
    scale: T,
    min: T,
    num: u16,
}

impl<T> Binning<T>
where
    T: Float,
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
    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.xaxis.num as usize
    }

    pub fn centers(&self) -> impl Iterator<Item = (T, T)> + '_ {
        self.yaxis
            .iter()
            .flat_map(move |y| self.xaxis.iter().zip(iter::repeat(y)))
    }

    pub fn fill(&mut self, i: usize, x: T, y: T) {
        let nx = self.xaxis.bin(x);
        let ny = self.yaxis.bin(y);
        if nx.is_none() || ny.is_none() {
            return;
        }
        let idx = self.index(nx.unwrap() as usize, ny.unwrap() as usize);
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
        let values = &mut [0, 0];
        let data: Vec<&mut [u32]> = vec![values];
        let mut histo = Histogram::new(0.0, 1.0, 2, 0.0, 1.0, 1, data);
        let centers: Vec<_> = histo.centers().collect();
        assert_eq!(centers, vec![(0.25, 0.5), (0.75, 0.5)]);
        histo.fill(0, -2.0, 3.0);
        histo.fill(0, 0.51, 0.1);
        assert_eq!(values[0], 0 as u32);
        assert_eq!(values[1], 1 as u32);
    }
}
