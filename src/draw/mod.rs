use std::{iter::Map, ops::AddAssign};

use log::warn;
use nalgebra::{RealField, Rotation2, Vector2};
use num::{traits::FloatConst, Float, FromPrimitive, ToPrimitive, Zero};

use crate::{
    units::{Absolute, Angle, Length, LengthType},
    Num,
};

use self::{
    coordinate::{Coordinate, LenCo},
    curve::{Bias, Split, SplitHalf},
};

pub mod coordinate;
pub mod curve;
pub(crate) mod transfer;

#[cfg(test)]
pub(crate) const APPROX_EQ_MARGIN: (f64, i64) = (0.000000000001, 1);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Resolution<T = Length<Absolute, f64>> {
    MinDistance(T),
    MinNumber(usize),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct _Arc<S> {
    radius: S,
}

impl<L, T> _Arc<Length<L, T>>
where
    L: LengthType,
    T: Float + FloatConst + Num + FromPrimitive,
{
    pub(crate) fn new(radius: Length<L, T>) -> Self {
        if radius.value.is_negative() {
            warn!("a negative radius {} is set", radius);
        }
        Self { radius }
    }
    pub(crate) fn radius(&self) -> Length<L, T> {
        self.radius
    }
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
    pub(crate) inner: _Arc<Length<L, T>>,
    pub(crate) center: Coordinate<Length<L, T>>,
    pub(crate) angle: (Angle<T>, Angle<T>),
    pub(crate) resolution: Resolution<Length<L, T>>,
}

impl<L, T> CircularArc<L, T>
where
    L: LengthType,
    T: Num + Float + FloatConst + FromPrimitive,
{
    pub fn new<C: Into<Coordinate<Length<L, T>>>>(
        radius: Length<L, T>,
        center: C,
        angle: (Angle<T>, Angle<T>),
        resolution: Resolution<Length<L, T>>,
    ) -> Self {
        Self {
            inner: _Arc::new(radius),
            center: center.into(),
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
    pub fn set_radius(&mut self, radius: Length<L, T>) -> &mut Self {
        self.inner = _Arc::new(radius);
        self
    }
    pub fn set_ang(&mut self, angle: (Angle<T>, Angle<T>)) -> &mut Self {
        self.angle = angle;
        self
    }
    pub fn set_center<C: Into<Coordinate<Length<L, T>>>>(&mut self, center: C) -> &mut Self {
        self.center = center.into();
        self
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
            .map(move |p| p + Vector2::from([self.center[0], self.center[1]]))
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
        let res = match self.resolution {
            Resolution::MinDistance(d) => Resolution::MinDistance(d),
            Resolution::MinNumber(n) => {
                let min_dis: Length<L, T> = (self.inner.radius
                    * (self.angle.1 - self.angle.0).to_rad().abs())
                    / T::from_usize(n).unwrap();
                Resolution::MinDistance(min_dis)
            }
        };
        (
            Self {
                angle: (self.angle.0, pos),
                resolution: res,
                ..self
            },
            Self {
                angle: (pos, self.angle.1),
                resolution: res,
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

impl<L, T> SplitHalf<Angle<T>> for CircularArc<L, T>
where
    L: LengthType,
    T: Num + Float + FloatConst + FromPrimitive,
{
    fn split_half(self) -> (Self, Self) {
        self.split((self.angle.0 + self.angle.1) / T::from_u8(2).unwrap())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line<L = Absolute, T = f64>
where
    L: LengthType,
    T: Num,
{
    pub start: LenCo<L, T>,
    pub end: LenCo<L, T>,
}

impl<L, T> Line<L, T>
where
    L: LengthType,
    T: Num,
{
    pub fn new<S: Into<LenCo<L, T>>, E: Into<LenCo<L, T>>>(sta: S, end: E) -> Self {
        Self {
            start: sta.into(),
            end: end.into(),
        }
    }
}

impl<L, T> IntoIterator for Line<L, T>
where
    L: LengthType,
    T: Num,
{
    type IntoIter = <[LenCo<L, T>; 2] as IntoIterator>::IntoIter;
    type Item = LenCo<L, T>;
    fn into_iter(self) -> Self::IntoIter {
        [self.start, self.end].into_iter()
    }
}

impl<L, T> Bias<Length<L, T>> for Line<L, T>
where
    L: LengthType,
    T: Num + RealField,
{
    fn bias(&mut self, b: Length<L, T>) {
        let mut v: Vector2<T> = Vector2::from(self.end.to_basic().0 - self.start.to_basic().0);
        v.normalize_mut();
        let t = Rotation2::new(T::frac_pi_2());
        let v: Vector2<T> = (t * v) * b.value;
        self.start = LenCo::from_basic(self.start.to_basic() + v);
        self.end = LenCo::from_basic(self.end.to_basic() + v);
    }
}

impl<L, T> Split<Length<L, T>> for Line<L, T>
where
    L: LengthType,
    T: Num + RealField,
{
    fn split(self, pos: Length<L, T>) -> (Self, Self) {
        let mid_pos: LenCo<L, T> = {
            let mut v: Vector2<T> = Vector2::from(self.end.to_basic().0 - self.start.to_basic().0);
            v.normalize_mut();
            LenCo::from_basic(self.start.to_basic() + v * pos.value)
        };
        (
            Self {
                start: self.start,
                end: mid_pos,
            },
            Self {
                start: mid_pos,
                end: self.end,
            },
        )
    }
}
impl<L, T> SplitHalf<Length<L, T>> for Line<L, T>
where
    L: LengthType,
    T: Num + RealField,
{
    fn split_half(self) -> (Self, Self) {
        let mid_pos = LenCo::from_basic(Coordinate::<T>::from(nalgebra::center(
            &self.start.to_basic().0,
            &self.end.to_basic().0,
        )));
        (
            Self {
                start: self.start,
                end: mid_pos,
            },
            Self {
                start: mid_pos,
                end: self.end,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::MILLIMETER;

    use super::*;
    use float_cmp::ApproxEq;

    #[test]
    fn bias_line() {
        let mut line = Line::new((MILLIMETER, MILLIMETER), (MILLIMETER * 2., MILLIMETER * 2.));
        line.bias(MILLIMETER * (2.).sqrt());
        assert!(line.start.approx_eq(
            LenCo::from((MILLIMETER * 0., MILLIMETER * 2.)),
            APPROX_EQ_MARGIN
        ));
        assert!(line.end.approx_eq(
            LenCo::from((MILLIMETER * 1., MILLIMETER * 3.)),
            APPROX_EQ_MARGIN
        ),);
        line.bias(MILLIMETER * (2.).sqrt() * (-2.));
        assert!(line.start.approx_eq(
            LenCo::from((MILLIMETER * 2., MILLIMETER * 0.)),
            APPROX_EQ_MARGIN
        ));
        assert!(line.end.approx_eq(
            LenCo::from((MILLIMETER * 3., MILLIMETER * 1.)),
            APPROX_EQ_MARGIN
        ),);
    }
    #[test]
    fn split_line() {
        let line = Line::new(
            (MILLIMETER * 0., MILLIMETER),
            (MILLIMETER * 0., MILLIMETER * 2.),
        );
        let (lower, upper) = line.clone().split(MILLIMETER / 3.);
        assert!(lower.start.approx_eq(line.start, APPROX_EQ_MARGIN));

        assert!(lower.end.approx_eq(
            LenCo::from((MILLIMETER * 0., MILLIMETER + MILLIMETER / 3.)),
            APPROX_EQ_MARGIN
        ));

        assert!(upper.start.approx_eq(
            LenCo::from((MILLIMETER * 0., MILLIMETER + MILLIMETER / 3.)),
            APPROX_EQ_MARGIN
        ));

        assert!(upper.end.approx_eq(line.end, APPROX_EQ_MARGIN));

        let (lower, upper) = line.clone().split_half();
        assert!(lower.start.approx_eq(line.start, APPROX_EQ_MARGIN));

        assert!(lower.end.approx_eq(
            LenCo::from((MILLIMETER * 0., MILLIMETER + MILLIMETER / 2.)),
            APPROX_EQ_MARGIN
        ));

        assert!(upper.start.approx_eq(
            LenCo::from((MILLIMETER * 0., MILLIMETER + MILLIMETER / 2.)),
            APPROX_EQ_MARGIN
        ));

        assert!(upper.end.approx_eq(line.end, APPROX_EQ_MARGIN));
    }
}
