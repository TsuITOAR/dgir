#![feature(type_alias_impl_trait)]
use gds21::GdsPoint;
use log::{info, warn};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};
use units::{AbsoluteLength, Length, LengthType};

pub mod color;
pub mod draw;
pub mod gds;
pub mod units;

pub trait Num:
    'static
    + Copy
    + Debug
    + Display
    + PartialOrd
    + num::traits::NumAssignRef
    + num::traits::Signed
    + num::traits::ToPrimitive
{
}

impl<T> Num for T where
    T: 'static
        + Copy
        + Debug
        + Display
        + PartialOrd
        + num::traits::NumAssignRef
        + num::traits::Signed
        + num::traits::ToPrimitive
{
}

pub trait Quantity: 'static + Clone + Debug + num::Zero + PartialEq + PartialOrd {}

impl<T> Quantity for T where T: 'static + Clone + Debug + num::Zero + PartialEq + PartialOrd {}

const MAX_POINTS_NUM: usize = 8191;

pub const NANOMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e-3,
    marker: PhantomData,
};

pub const MICROMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e0,
    marker: PhantomData,
};

pub const MILLIMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e3,
    marker: PhantomData,
};

pub const CENTIMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e4,
    marker: PhantomData,
};

pub const METER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e6,
    marker: PhantomData,
};

pub fn zero<L: LengthType, T: Num>() -> Length<L, T> {
    Length {
        value: num::Zero::zero(),
        marker: PhantomData,
    }
}

fn points_num_check(points: &Vec<GdsPoint>) -> bool {
    if points.len() > MAX_POINTS_NUM {
        warn!(
            "points number({}) exceeds limit({})",
            points.len(),
            MAX_POINTS_NUM
        );
        return false;
    }
    return true;
}

fn close_curve(points: &mut Vec<GdsPoint>) -> bool {
    if points.len() >= 2 && points.first() == points.last() {
        warn!(
            "curve not closed, start at ({}, {}), end at ({}, {})",
            points.first().unwrap().x,
            points.first().unwrap().y,
            points.last().unwrap().x,
            points.last().unwrap().y
        );
        false
    } else {
        true
    }
}

fn split_polygon<T: Clone>(mut raw: Vec<T>, max_points: usize) -> Vec<Vec<T>> {
    let len = raw.len();
    if raw.len() > max_points {
        info!("auto splitting polygon");
        let mut ret = Vec::new();
        let mut temp = Vec::with_capacity(len / 2 + 1);
        temp.push(raw[len / 4].clone());
        temp.extend(raw.drain(len / 4 + 1..3 * len / 4));
        temp.push(raw[len / 4 + 1].clone());
        ret.extend(split_polygon(temp, max_points));
        ret.extend(split_polygon(raw, max_points));
        ret
    } else {
        vec![raw]
    }
}

fn split_path<T: Clone>(mut raw: Vec<T>, max_points: usize) -> Vec<Vec<T>> {
    let len = raw.len();
    if raw.len() > max_points {
        info!("auto splitting path");
        let mut ret = Vec::new();
        let mut temp = Vec::with_capacity(len / 2 + 1);
        temp.extend(raw.drain(0..len / 2));
        temp.push(raw[0].clone());
        ret.extend(split_path(temp, max_points));
        ret.extend(split_path(raw, max_points));
        ret
    } else {
        vec![raw]
    }
}

#[cfg(test)]
mod tests {
    use crate::split_path;

    use super::split_polygon;
    #[test]
    fn auto_split_polygon() {
        let v = (0..10).collect::<Vec<_>>();
        let r = split_polygon(v, 2);
        assert_eq!(r.len(), 6);
    }
    #[test]
    fn auto_split_path() {
        let v = (0..10).collect::<Vec<_>>();
        let r = split_path(v, 2);
        assert_eq!(r.len(), 6);
    }
}
