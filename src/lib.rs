#![feature(type_alias_impl_trait)]
use gds21::GdsPoint;
use log::warn;
use nalgebra::Scalar;
use num::Num;
use std::marker::PhantomData;
use units::{AbsoluteLength, Length, LengthType};

pub mod color;
pub mod draw;
pub mod gds;
pub mod units;
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

pub fn zero<L: LengthType, T: Scalar + Num>() -> Length<L, T> {
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
