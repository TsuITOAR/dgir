use core::f64;
use std::{
    marker::PhantomData,
    ops::{Add, Div, Mul, Sub},
};

use num::{FromPrimitive, Num};

pub trait AbsoluteUnit<S: Num + FromPrimitive>: Copy {
    const CONVERSION_FACTOR: f64;
}

pub trait RelativeUnit<S: Num + FromPrimitive>: Copy {
    const CONVERSION_FACTOR: f64;
}
#[derive(Clone, Debug)]
pub struct MakeLength<U, S: Num = f64> {
    marker: PhantomData<(U, S)>,
}

impl<U, S: Num + FromPrimitive> MakeLength<U, S> {
    pub fn new_absolute(value: S) -> Length<AbsoluteLength<S>, S>
    where
        U: AbsoluteUnit<S>,
    {
        Length {
            value: value * S::from_f64(<U as AbsoluteUnit<S>>::CONVERSION_FACTOR).unwrap(),
            marker: PhantomData,
        }
    }
    pub fn new_relative(value: S) -> Length<RelativeLength<S>, S>
    where
        U: RelativeUnit<S>,
    {
        Length {
            value: value * S::from_f64(<U as RelativeUnit<S>>::CONVERSION_FACTOR).unwrap(),
            marker: PhantomData,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Length<T: LengthType<S>, S> {
    value: S,
    marker: PhantomData<T>,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct AbsoluteLength<S> {
    marker: PhantomData<S>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RelativeLength<S> {
    marker: PhantomData<S>,
}

pub(crate) trait LengthType<S> {}
impl<S> LengthType<S> for AbsoluteLength<S> {}
impl<S> LengthType<S> for RelativeLength<S> {}

impl<U: AbsoluteUnit<S> + FromPrimitive, S: Num + Copy + FromPrimitive> Copy for MakeLength<U, S> {}

impl<T: LengthType<S>, S: Num> Add<Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn add(mut self, rhs: Length<T, S>) -> Self::Output {
        self.value = self.value + rhs.value;
        self
    }
}

impl<T: LengthType<S>, S: Num> Sub<Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn sub(mut self, rhs: Length<T, S>) -> Self::Output {
        self.value = self.value - rhs.value;
        self
    }
}

impl<T: LengthType<S>, S: Num> Mul<S> for Length<T, S> {
    type Output = Length<T, S>;
    fn mul(mut self, rhs: S) -> Self::Output {
        self.value = self.value * rhs;
        self
    }
}

impl<T: LengthType<S>, S: Num> Div<Length<T, S>> for Length<T, S> {
    type Output = S;
    fn div(self, rhs: Length<T, S>) -> Self::Output {
        self.value / rhs.value
    }
}

impl<T: LengthType<S>, S: Num + PartialEq> PartialEq<Length<T, S>> for Length<T, S> {
    fn eq(&self, other: &Length<T, S>) -> bool {
        self.value == other.value
    }
}

impl<T: LengthType<S>, S: Num + PartialOrd> PartialOrd<Length<T, S>> for Length<T, S> {
    fn partial_cmp(&self, other: &Length<T, S>) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Nanometer;
impl<S:FromPrimitive+Num> AbsoluteUnit<S> for Nanometer {
    const CONVERSION_FACTOR: f64 = 1e-3;
}

#[derive(Debug, Clone, Copy)]
pub struct Micrometer;
impl<S:FromPrimitive+Num> AbsoluteUnit<S> for Micrometer {
    const CONVERSION_FACTOR: f64 = 1e0;
}

#[derive(Debug, Clone, Copy)]
pub struct Millimeter;
impl<S:FromPrimitive+Num> AbsoluteUnit<S> for Millimeter {
    const CONVERSION_FACTOR: f64 = 1e3;
}

#[derive(Debug, Clone, Copy)]
pub struct Centimeter;
impl<S: FromPrimitive + Num> AbsoluteUnit<S> for Centimeter {
    const CONVERSION_FACTOR: f64 = 1e4;
}

#[derive(Debug, Clone, Copy)]
pub struct Meter;
impl<S: FromPrimitive + Num> AbsoluteUnit<S> for Meter {
    const CONVERSION_FACTOR: f64 = 1e6;
}
#[derive(Debug, Clone, Copy)]
pub struct UserUnit;
impl<S: FromPrimitive + Num> RelativeUnit<S> for UserUnit {
    const CONVERSION_FACTOR: f64 = 1.;
}

#[test]
fn AbsoluteUnits_operation() {
    let l1 = MakeLength::<Nanometer>::new_absolute(1000.);
    let l2 = MakeLength::<Micrometer>::new_absolute(1.);
    let l3 = MakeLength::<Millimeter>::new_absolute(1.);
    assert_eq!(l1, l2);
    assert_eq!(l3, l2 * 1000.);
    assert_eq!(l3, (l1 + l2) * 500.);
    assert!(l1 < l3);
    assert_eq!(l1 / l2, 1.);
}
