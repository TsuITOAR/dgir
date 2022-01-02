use core::f64;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use num::{
    traits::{FloatConst, NumRef},
    Float, FromPrimitive, Num, Zero,
};

pub trait AbsoluteUnit {
    const CONVERSION_FACTOR: f64;
}

pub trait RelativeUnit {
    const CONVERSION_FACTOR: u32;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Length<T: LengthType, S> {
    pub(crate) value: S,
    pub(crate) marker: PhantomData<T>,
}

impl<T: LengthType, S: Display> Display for Length<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<M, T, S> float_cmp::ApproxEq for Length<T, S>
where
    M: Copy + Default,
    T: LengthType,
    S: float_cmp::ApproxEq<Margin = M>,
{
    type Margin = M;
    fn approx_eq<N: Into<Self::Margin>>(self, other: Self, margin: N) -> bool {
        let margin = margin.into();
        self.value.approx_eq(other.value, margin)
    }
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
            value: value * S::from_u32(<U as RelativeUnit>::CONVERSION_FACTOR).unwrap(),
            marker: PhantomData,
        }
    }
}
impl<T: LengthType, S: Neg<Output = S>> Neg for Length<T, S> {
    type Output = Length<T, S>;
    fn neg(self) -> Self::Output {
        Length::<T, S> {
            value: self.value.neg(),
            marker: PhantomData,
        }
    }
}

impl<T: LengthType, S: Num> Add<Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn add(self, rhs: Length<T, S>) -> Self::Output {
        Self {
            value: self.value + rhs.value,
            marker: self.marker,
        }
    }
}

impl<T: LengthType, S: AddAssign> AddAssign<Length<T, S>> for Length<T, S> {
    fn add_assign(&mut self, rhs: Length<T, S>) {
        self.value += rhs.value;
    }
}

impl<T: LengthType, S: Num> Sub<Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn sub(self, rhs: Length<T, S>) -> Self::Output {
        Self {
            value: self.value - rhs.value,
            marker: self.marker,
        }
    }
}

impl<T: LengthType, S: SubAssign> SubAssign<Length<T, S>> for Length<T, S> {
    fn sub_assign(&mut self, rhs: Length<T, S>) {
        self.value -= rhs.value;
    }
}

impl<T: LengthType, S: Num> Mul<S> for Length<T, S> {
    type Output = Length<T, S>;
    fn mul(self, rhs: S) -> Self::Output {
        Self {
            value: self.value * rhs,
            marker: self.marker,
        }
    }
}

impl<T: LengthType> Mul<Length<T, f64>> for f64 {
    type Output = Length<T, f64>;
    fn mul(self, rhs: Length<T, f64>) -> Self::Output {
        Self::Output {
            value: self * rhs.value,
            ..rhs
        }
    }
}

impl<T: LengthType> Mul<Length<T, f32>> for f32 {
    type Output = Length<T, f32>;
    fn mul(self, rhs: Length<T, f32>) -> Self::Output {
        Self::Output {
            value: self * rhs.value,
            ..rhs
        }
    }
}

impl<T: LengthType, S: MulAssign> MulAssign<S> for Length<T, S> {
    fn mul_assign(&mut self, rhs: S) {
        self.value *= rhs;
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

impl<T: LengthType, S: Num + DivAssign> DivAssign<S> for Length<T, S> {
    fn div_assign(&mut self, rhs: S) {
        self.value /= rhs;
    }
}

impl<'a, T: LengthType, S: NumRef> Add<&'a Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn add(self, rhs: &'a Length<T, S>) -> Self::Output {
        Self {
            value: self.value + &rhs.value,
            marker: self.marker,
        }
    }
}

impl<'a, T: LengthType, S: for<'r> AddAssign<&'r S>> AddAssign<&'a Length<T, S>> for Length<T, S> {
    fn add_assign(&mut self, rhs: &'a Length<T, S>) {
        self.value += &rhs.value;
    }
}

impl<'a, T: LengthType, S: NumRef> Sub<&'a Length<T, S>> for Length<T, S> {
    type Output = Length<T, S>;
    fn sub(self, rhs: &'a Length<T, S>) -> Self::Output {
        Self {
            value: self.value - &rhs.value,
            marker: self.marker,
        }
    }
}

impl<'a, T: LengthType, S: for<'r> SubAssign<&'r S>> SubAssign<&'a Length<T, S>> for Length<T, S> {
    fn sub_assign(&mut self, rhs: &'a Length<T, S>) {
        self.value -= &rhs.value;
    }
}

impl<'a, T: LengthType, S: NumRef> Mul<&'a S> for Length<T, S> {
    type Output = Length<T, S>;
    fn mul(self, rhs: &'a S) -> Self::Output {
        Self {
            value: self.value * rhs,
            marker: PhantomData,
        }
    }
}

impl<'a, T: LengthType> Mul<&'a Length<T, f64>> for f64 {
    type Output = Length<T, f64>;
    fn mul(self, rhs: &'a Length<T, f64>) -> Self::Output {
        Self::Output {
            value: self * rhs.value,
            marker: PhantomData,
        }
    }
}

impl<'a, T: LengthType> Mul<&'a Length<T, f32>> for f32 {
    type Output = Length<T, f32>;
    fn mul(self, rhs: &'a Length<T, f32>) -> Self::Output {
        Self::Output {
            value: self * rhs.value,
            marker: PhantomData,
        }
    }
}

impl<'a, T: LengthType, S: for<'r> MulAssign<&'r S>> MulAssign<&'a S> for Length<T, S> {
    fn mul_assign(&mut self, rhs: &'a S) {
        self.value *= rhs;
    }
}

impl<'a, T: LengthType, S: NumRef> Div<&'a Length<T, S>> for Length<T, S> {
    type Output = S;
    fn div(self, rhs: &'a Length<T, S>) -> Self::Output {
        self.value / &rhs.value
    }
}

impl<'a, T: LengthType, S: NumRef> Div<&'a S> for Length<T, S> {
    type Output = Length<T, S>;
    fn div(self, rhs: &'a S) -> Self::Output {
        Self {
            value: self.value / rhs,
            marker: self.marker,
        }
    }
}

impl<'a, T: LengthType, S: for<'r> DivAssign<&'r S>> DivAssign<&'a S> for Length<T, S> {
    fn div_assign(&mut self, rhs: &'a S) {
        self.value /= rhs;
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

impl<U: LengthType, S: Num> Zero for Length<U, S> {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Absolute;

#[derive(Debug, Clone, Copy, Default)]
pub struct Relative;

pub trait LengthType: 'static + Clone + Copy + Debug {}
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
pub struct DbUnit;
impl RelativeUnit for DbUnit {
    const CONVERSION_FACTOR: u32 = 1;
}

pub type AbsoluteLength<S> = Length<Absolute, S>;
pub type RelativeLength<S> = Length<Relative, S>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Deg<S>(S);

#[derive(Debug, Clone, Copy, Default)]
pub struct Rad<S>(S);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
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
    pub fn cos(self) -> S
    where
        S: Float,
    {
        self.to_rad().cos()
    }
    pub fn sin(self) -> S
    where
        S: Float,
    {
        self.to_rad().sin()
    }
}

impl<S: Num + Display> Display for Angle<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}rad", self.0)
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

impl<S: Div<Output = S>> Div<Angle<S>> for Angle<S> {
    type Output = S;
    fn div(self, rhs: Angle<S>) -> Self::Output {
        self.0 / rhs.0
    }
}

impl<S: Neg<Output = S>> Neg for Angle<S> {
    type Output = Angle<S>;
    fn neg(self) -> Self::Output {
        Angle::<S>(self.0.neg())
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
    let deg = Angle::from_deg(180.);
    let ang = Angle::from_rad(f64::PI());
    assert_eq!(deg, ang);
    assert_eq!(deg + ang, Angle::from_deg(360.));
    assert_eq!(deg - ang, Angle::from_rad(0.));
}
