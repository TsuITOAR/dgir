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

impl<U: Unit<S>, S: Num> Length<U, S> {
    pub fn new(value: S) -> Self {
        Self {
            value: value * <U as Unit<S>>::CONVERSION_FACTOR,
            units: PhantomData,
        }
    }
    pub(crate) fn conversion<V: Unit<S>>(self) -> Length<V, S> {
        Length::<V, S> {
            value: self.value,
            units: PhantomData,
        }
    }
    fn set_unit<V: Unit<S>>(self, _unit: V) -> Length<V, S> {
        Length::<V, S> {
            value: self.value,
            units: PhantomData,
        }
    }
}
impl<U: Unit<S>, S: Num + Copy> Copy for Length<U, S> {}

impl<U: Unit<S>, V: Unit<S>, S: Num> Add<Length<V, S>> for Length<U, S> {
    type Output = Length<U, S>;
    fn add(mut self, rhs: Length<V, S>) -> Self::Output {
        self.value = self.value + rhs.value;
        self
    }
}

impl<U: Unit<S>, V: Unit<S>, S: Num> Sub<Length<V, S>> for Length<U, S> {
    type Output = Length<U, S>;
    fn sub(mut self, rhs: Length<V, S>) -> Self::Output {
        self.value = self.value - rhs.value;
        self
    }
}

impl<U: Unit<S>, S: Num> Mul<S> for Length<U, S> {
    type Output = Length<U, S>;
    fn mul(mut self, rhs: S) -> Self::Output {
        self.value = self.value * rhs;
        self
    }
}

impl<U: Unit<S>, V: Unit<S>, S: Num> Div<Length<V, S>> for Length<U, S> {
    type Output = S;
    fn div(self, rhs: Length<V, S>) -> Self::Output {
        self.value / rhs.value
    }
}

impl<U: Unit<S>, V: Unit<S>, S: Num + PartialEq> PartialEq<Length<V, S>> for Length<U, S> {
    fn eq(&self, other: &Length<V, S>) -> bool {
        self.value == other.value
    }
}

impl<U: Unit<S>, V: Unit<S>, S: Num + PartialOrd> PartialOrd<Length<V, S>> for Length<U, S> {
    fn partial_cmp(&self, other: &Length<V, S>) -> Option<std::cmp::Ordering> {
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
    assert!(l1 < l3);
    assert_eq!(l1 / l2, 1.);
}
