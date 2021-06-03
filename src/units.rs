use core::f64;
use std::{
    marker::PhantomData,
    ops::{Add, Div, Mul, Sub},
};

use num::{traits::FloatConst, Float, FromPrimitive, Num, ToPrimitive, Zero};
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

pub trait Angle {
    type Basic;
    fn to_rad(self) -> Self::Basic;
    fn to_deg(self) -> Self::Basic;
    fn from_rad(rad: Self::Basic) -> Self;
    fn from_deg(deg: Self::Basic) -> Self;
}

impl<S> Angle for Deg<S>
where
    S: FloatConst + Float + Zero + FromPrimitive + ToPrimitive,
{
    type Basic = S;
    fn to_deg(self) -> Self::Basic {
        self.0
    }
    fn to_rad(self) -> Self::Basic {
        self.0 / <Self::Basic as FromPrimitive>::from_f64(180.).unwrap()
            * <Self::Basic as FloatConst>::PI()
    }
    fn from_deg(deg: Self::Basic) -> Self {
        Self(deg)
    }
    fn from_rad(rad: Self::Basic) -> Self {
        Self(
            rad * <Self::Basic as FromPrimitive>::from_f64(180.).unwrap()
                / <Self::Basic as FloatConst>::PI(),
        )
    }
}

impl<S> Angle for Rad<S>
where
    S: FloatConst + Float + Zero + FromPrimitive + ToPrimitive,
{
    type Basic = S;
    fn to_deg(self) -> Self::Basic {
        self.0 / <Self::Basic as FloatConst>::PI()
            * <Self::Basic as FromPrimitive>::from_f64(180.).unwrap()
    }
    fn to_rad(self) -> Self::Basic {
        self.0
    }
    fn from_deg(deg: Self::Basic) -> Self {
        Self(
            deg * <Self::Basic as FloatConst>::PI()
                / <Self::Basic as FromPrimitive>::from_f64(180.).unwrap(),
        )
    }
    fn from_rad(rad: Self::Basic) -> Self {
        Self(rad)
    }
}

impl<S> From<Rad<S>> for Deg<S>
where
    S: FloatConst + Float + Zero + FromPrimitive + ToPrimitive,
{
    fn from(rad: Rad<S>) -> Self {
        Self(rad.to_deg())
    }
}

impl<S> From<Deg<S>> for Rad<S>
where
    S: FloatConst + Float + Zero + FromPrimitive + ToPrimitive,
{
    fn from(deg: Deg<S>) -> Self {
        Self(deg.to_rad())
    }
}

impl<S: Add<Output = S>, T: Angle<Basic = S>> Add<T> for Rad<S> {
    type Output = Rad<S>;
    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs.to_rad())
    }
}

impl<S: Add<Output = S>, T: Angle<Basic = S>> Add<T> for Deg<S> {
    type Output = Deg<S>;
    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs.to_deg())
    }
}

impl<S: Sub<Output = S>, T: Angle<Basic = S>> Sub<T> for Rad<S> {
    type Output = Rad<S>;
    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs.to_rad())
    }
}

impl<S: Sub<Output = S>, T: Angle<Basic = S>> Sub<T> for Deg<S> {
    type Output = Deg<S>;
    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs.to_deg())
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
