use std::{iter::Map, ops::AddAssign};

use log::warn;
use nalgebra::Vector2;
use num::{traits::FloatConst, Float, FromPrimitive, ToPrimitive, Zero};

use crate::{
    units::{Absolute, Angle, Length, LengthType},
    Num,
};

use self::{
    coordinate::{Coordinate, LenCo},
    curve::{Bias, Split},
};

pub mod coordinate;
pub mod curve;
pub mod transfer;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Resolution<T = Length<Absolute, f64>> {
    MinDistance(T),
    MinNumber(usize),
}

#[derive(Debug, Clone, Copy)]
struct _Arc<S> {
    radius: S,
}

impl<L, T> _Arc<Length<L, T>>
where
    L: LengthType,
    T: Float + FloatConst + Num + FromPrimitive,
{
    fn to_points(
        self,
        angle: (Angle<T>, Angle<T>),
        resolution: Resolution<Length<L, T>>,
    ) -> Map<std::ops::RangeInclusive<usize>, impl FnMut(usize) -> LenCo<L, T>> {
        let ang_range = (angle.1 - angle.0).to_rad();
        let section_num = match resolution {
            Resolution::MinNumber(n) => {
                debug_assert!(n > 1);
                n - 1
            }
            Resolution::MinDistance(d) => (ang_range.abs() * (self.radius / d).abs())
                .to_usize()
                .unwrap(),
        };
        let ang_at = move |s: usize| {
            debug_assert!(s <= section_num);
            ang_range / FromPrimitive::from_usize(section_num).unwrap()
                * FromPrimitive::from_usize(s).unwrap()
                + angle.0.to_rad()
        };
        (0..=section_num).into_iter().map(move |x| {
            Coordinate::from([self.radius * ang_at(x).cos(), self.radius * ang_at(x).sin()])
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CircularArc<L = Absolute, T = f64>
where
    L: LengthType,
    T: Num,
{
    inner: _Arc<Length<L, T>>,
    center: (Length<L, T>, Length<L, T>),
    angle: (Angle<T>, Angle<T>),
    resolution: Resolution<Length<L, T>>,
}

impl<L, T> CircularArc<L, T>
where
    L: LengthType,
    T: Num,
{
    pub fn new(
        radius: Length<L, T>,
        center: (Length<L, T>, Length<L, T>),
        angle: (Angle<T>, Angle<T>),
        resolution: Resolution<Length<L, T>>,
    ) -> Self {
        Self {
            inner: _Arc { radius },
            center,
            angle,
            resolution,
        }
    }
    pub fn new_origin(
        radius: Length<L, T>,
        angle: (Angle<T>, Angle<T>),
        resolution: Resolution<Length<L, T>>,
    ) -> Self {
        Self::new(radius, (Zero::zero(), Zero::zero()), angle, resolution)
    }
}

impl<L, T> IntoIterator for CircularArc<L, T>
where
    L: LengthType,
    T: Float + FloatConst + Num + FromPrimitive + AddAssign,
{
    type IntoIter = impl DoubleEndedIterator<Item = LenCo<L, T>>;
    type Item = LenCo<L, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner
            .to_points(self.angle, self.resolution)
            .map(move |p| p + Vector2::from([self.center.0, self.center.1]))
    }
}

impl<L, T> Bias<Length<L, T>> for CircularArc<L, T>
where
    L: LengthType,
    T: Float + FloatConst + Num + FromPrimitive + AddAssign,
{
    fn bias(&mut self, b: Length<L, T>) {
        self.inner.radius += b;
    }
}

impl<L, T> Split<Angle<T>> for CircularArc<L, T>
where
    L: LengthType,
    T: Num + FromPrimitive + ToPrimitive,
{
    fn split(self, pos: Angle<T>) -> (Self, Self) {
        if (pos > self.angle.0 && pos > self.angle.1) || (pos < self.angle.0 && pos < self.angle.1)
        {
            warn!(
                "split position at {}, but the original arc start at {}, end at {}",
                pos, self.angle.0, self.angle.1
            );
        }
        let (res_left, res_right) = match self.resolution {
            Resolution::MinDistance(d) => (Resolution::MinDistance(d), Resolution::MinDistance(d)),
            Resolution::MinNumber(n) => {
                let min_dis: Length<L, T> = (self.inner.radius
                    * (self.angle.1 - self.angle.0).to_rad().abs())
                    / T::from_usize(n).unwrap();
                (
                    Resolution::MinDistance(min_dis),
                    Resolution::MinDistance(min_dis),
                )
            }
        };
        (
            Self {
                angle: (self.angle.0, self.angle.0 + pos),
                resolution: res_left,
                ..self
            },
            Self {
                angle: (self.angle.0 + pos, self.angle.1),
                resolution: res_right,
                ..self
            },
        )
    }
}

impl<L, T> Split<Length<L, T>> for CircularArc<L, T>
where
    L: LengthType,
    T: Num + Float + FloatConst + FromPrimitive,
{
    fn split(self, pos: Length<L, T>) -> (Self, Self) {
        let angle = Angle::from_deg(pos / self.inner.radius);
        self.split(angle)
    }
}
