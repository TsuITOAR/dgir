use core::f64;
use std::{
    marker::PhantomData,
    ops::{Add, Div, Mul, Sub},
};

use num::Num;

pub trait Unit<S: Num>: Copy {
    const CONVERSION_FACTOR: S;
}
#[derive(Clone, Debug)]
pub struct Length<U: Unit<S>, S: Num = f64> {
    pub value: S,
    units: PhantomData<U>,
}

impl<U: Unit<f64>> Length<U, f64> {
    fn new(value: f64) -> Self {
        Self {
            value: value * <U as Unit<f64>>::CONVERSION_FACTOR,
            units: PhantomData,
        }
    }
    fn conversion<V: Unit<f64>>(self) -> Length<V, f64> {
        Length::<V, f64> {
            value: self.value,
            units: PhantomData,
        }
    }
    fn set_unit<V: Unit<f64>>(self, _unit: V) -> Length<V, f64> {
        Length::<V, f64> {
            value: self.value,
            units: PhantomData,
        }
    }
}
impl<U: Unit<V>, V: Num + Copy> Copy for Length<U, V> {}

impl<U: Unit<f64>, V: Unit<f64>> Add<Length<V>> for Length<U, f64> {
    type Output = Length<U>;
    fn add(mut self, rhs: Length<V>) -> Self::Output {
        self.value += rhs.value;
        self
    }
}

impl<U: Unit<f64>, V: Unit<f64>> Sub<Length<V>> for Length<U, f64> {
    type Output = Length<U>;
    fn sub(mut self, rhs: Length<V>) -> Self::Output {
        self.value -= rhs.value;
        self
    }
}

impl<U: Unit<f64>> Mul<f64> for Length<U, f64> {
    type Output = Length<U>;
    fn mul(mut self, rhs: f64) -> Self::Output {
        self.value *= rhs;
        self
    }
}

impl<U: Unit<f64>, V: Unit<f64>> Div<Length<V>> for Length<U, f64> {
    type Output = f64;
    fn div(self, rhs: Length<V>) -> Self::Output {
        self.value / rhs.value
    }
}

impl<U: Unit<f64>, V: Unit<f64>> PartialEq<Length<V>> for Length<U, f64> {
    fn eq(&self, other: &Length<V>) -> bool {
        self.value == other.value
    }
}

impl<U: Unit<f64>, V: Unit<f64>> PartialOrd<Length<V>> for Length<U, f64> {
    fn partial_cmp(&self, other: &Length<V>) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Nanometer;
impl Unit<f64> for Nanometer {
    const CONVERSION_FACTOR: f64 = 1e-3;
}

#[derive(Debug, Clone, Copy)]
pub struct Micrometer;
impl Unit<f64> for Micrometer {
    const CONVERSION_FACTOR: f64 = 1e0;
}

#[derive(Debug, Clone, Copy)]
pub struct Millimeter;
impl Unit<f64> for Millimeter {
    const CONVERSION_FACTOR: f64 = 1e3;
}

#[derive(Debug, Clone, Copy)]
pub struct Centimeter;
impl Unit<f64> for Centimeter {
    const CONVERSION_FACTOR: f64 = 1e4;
}

#[test]
fn units_operation() {
    let l1 = Length::<Nanometer>::new(1000.);
    let l2 = Length::<Micrometer>::new(1.);
    let l3 = Length::<Millimeter>::new(1.);
    assert_eq!(l1, l2);
    assert_eq!(l3, l2 * 1000.);
    assert_eq!(l3, (l1 + l2) * 500.);
}
