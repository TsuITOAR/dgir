use core::f64;
use std::{
    marker::PhantomData,
    ops::{Add, Div, Mul, Neg, Sub},
};

use num::{traits::FloatConst, FromPrimitive, Num, Zero};

pub trait AbsoluteUnit {
    const CONVERSION_FACTOR: f64;
}

pub trait RelativeUnit {
    const CONVERSION_FACTOR: f64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Length<T: LengthType, S> {
    pub(crate) value: S,
    pub(crate) marker: PhantomData<T>,
}

impl<S> Length<Absolute, S> {
    pub fn new_absolute<U>(value: S) -> Length<Absolute, S>
    where
        S: Num + FromPrimitive,
        U: AbsoluteUnit,
    {
        Length {
            value: value * S::from_f64(<U as AbsoluteUnit>::CONVERSION_FACTOR).unwrap(),
            marker: PhantomData,
        }
    }
}
impl<S> Length<Relative, S> {
    pub fn new_relative<U>(value: S) -> Length<Relative, S>
    where
        S: Num + FromPrimitive,
        U: RelativeUnit,
    {
        Length {
            value: value * S::from_f64(<U as RelativeUnit>::CONVERSION_FACTOR).unwrap(),
            marker: PhantomData,
        }
    }
}
impl<T: LengthType, S: Neg<Output = S>> Neg for Length<T, S> {
    type Output = Length<T, S>;
    fn neg(self) -> Self::Output {
        Length::<T, S> {
            value: -self.value,
            marker: PhantomData,
        }
    }
}

impl<T: LengthType, S: Num> Add<Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn add(mut self, rhs: Length<T, S>) -> Self::Output {
        self.value = self.value + rhs.value;
        self
    }
}

impl<T: LengthType, S: Num> Sub<Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn sub(mut self, rhs: Length<T, S>) -> Self::Output {
        self.value = self.value - rhs.value;
        self
    }
}

impl<T: LengthType, S: Num> Mul<S> for Length<T, S> {
    type Output = Length<T, S>;
    fn mul(mut self, rhs: S) -> Self::Output {
        self.value = self.value * rhs;
        self
    }
}

impl<T: LengthType, S: Num> Div<Length<T, S>> for Length<T, S> {
    type Output = S;
    fn div(self, rhs: Length<T, S>) -> Self::Output {
        self.value / rhs.value
    }
}
impl<T: LengthType, S: Num> Div<S> for Length<T, S> {
    type Output = Length<T, S>;
    fn div(self, rhs: S) -> Self::Output {
        Self {
            value: self.value / rhs,
            marker: self.marker,
        }
    }
}

impl<T: LengthType, S: Num + PartialEq> PartialEq<Length<T, S>> for Length<T, S> {
    fn eq(&self, other: &Length<T, S>) -> bool {
        self.value == other.value
    }
}

impl<T: LengthType, S: Num + PartialOrd> PartialOrd<Length<T, S>> for Length<T, S> {
    fn partial_cmp(&self, other: &Length<T, S>) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<U: LengthType, S: Zero + Num> Zero for Length<U, S> {
    fn zero() -> Self {
        Length {
            value: S::zero(),
            marker: PhantomData,
        }
    }
    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Absolute;

#[derive(Debug, Clone, Copy)]
pub struct Relative;

pub trait LengthType {}
impl LengthType for Absolute {}
impl LengthType for Relative {}
#[derive(Debug, Clone, Copy)]
pub struct Nanometer;
impl AbsoluteUnit for Nanometer {
    const CONVERSION_FACTOR: f64 = 1e-3;
}

#[derive(Debug, Clone, Copy)]
pub struct Micrometer;
impl AbsoluteUnit for Micrometer {
    const CONVERSION_FACTOR: f64 = 1e0;
}

#[derive(Debug, Clone, Copy)]
pub struct Millimeter;
impl AbsoluteUnit for Millimeter {
    const CONVERSION_FACTOR: f64 = 1e3;
}

#[derive(Debug, Clone, Copy)]
pub struct Centimeter;
impl AbsoluteUnit for Centimeter {
    const CONVERSION_FACTOR: f64 = 1e4;
}

#[derive(Debug, Clone, Copy)]
pub struct Meter;
impl AbsoluteUnit for Meter {
    const CONVERSION_FACTOR: f64 = 1e6;
}
#[derive(Debug, Clone, Copy)]
pub struct UserUnit;
impl RelativeUnit for UserUnit {
    const CONVERSION_FACTOR: f64 = 1.;
}

pub type AbsoluteLength<S> = Length<Absolute, S>;
pub type RelativeLength<S> = Length<Relative, S>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Deg<S>(S);

#[derive(Debug, Clone, Copy, Default)]
pub struct Rad<S>(S);

#[derive(Debug, Clone, Copy)]
pub struct Angle<S>(S);
impl<S> Angle<S> {
    pub fn to_rad(self) -> S {
        self.0
    }
    pub fn to_deg(self) -> S
    where
        S: Num + FloatConst + FromPrimitive,
    {
        self.0 / S::PI() * S::from_f64(180.).unwrap()
    }
    pub fn from_rad(rad: S) -> Self {
        Self(rad)
    }
    pub fn from_deg(deg: S) -> Self
    where
        S: Num + FloatConst + FromPrimitive,
    {
        Self(deg / S::from_f64(180.).unwrap() * S::PI())
    }
}

impl<S: Add<Output = S>> Add<Angle<S>> for Angle<S> {
    type Output = Angle<S>;
    fn add(self, rhs: Angle<S>) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<S: Sub<Output = S>> Sub<Angle<S>> for Angle<S> {
    type Output = Angle<S>;
    fn sub(self, rhs: Angle<S>) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<S: Mul<Output = S>> Mul<S> for Angle<S> {
    type Output = Angle<S>;
    fn mul(self, rhs: S) -> Self::Output {
        Angle::<S>(self.0 * rhs)
    }
}

impl<S: Div<Output = S>> Div<S> for Angle<S> {
    type Output = Angle<S>;
    fn div(self, rhs: S) -> Self::Output {
        Angle::<S>(self.0 / rhs)
    }
}

impl<S: Neg<Output = S>> Neg for Angle<S> {
    type Output = Angle<S>;
    fn neg(self) -> Self::Output {
        Angle::<S>(-self.0)
    }
}

#[test]
fn units_operation() {
    let l1 = Length::new_absolute::<Nanometer>(1000.);
    let l2 = Length::new_absolute::<Micrometer>(1.);
    let l3 = Length::new_absolute::<Millimeter>(1.);
    assert_eq!(l1, l2);
    assert_eq!(l3, l2 * 1000.);
    assert_eq!(l3, (l1 + l2) * 500.);
    assert!(l1 < l3);
    assert_eq!(l1 / l2, 1.);
}
