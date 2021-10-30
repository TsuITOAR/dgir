use super::{Coordinate, Distance, Resolution};
use crate::units::Angle;
use nalgebra::Scalar;
use num::{traits::FloatConst, Float, FromPrimitive, ToPrimitive};
use std::{
    fmt::Debug,
    iter::{Chain, Fuse, FusedIterator, Iterator, Map, Rev},
};
pub struct Curve<T: Scalar, C: Iterator<Item = Coordinate<T>>> {
    curve: C,
}

impl<T: Scalar, C: Iterator<Item = Coordinate<T>>> Curve<T, C> {
    pub fn new(curve: C) -> Self {
        Self { curve }
    }
    pub fn close(self) -> Area<T, Close<T, Fuse<Self>>>
    where
        T: Copy + PartialEq,
        C: Iterator<Item = Coordinate<T>>,
    {
        Area::new(Close {
            curve: self.fuse(),
            first: None,
            current: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Close<T: Scalar, C: FusedIterator<Item = Coordinate<T>>> {
    curve: C,
    first: Option<C::Item>,
    current: Option<C::Item>,
}

impl<T: Scalar, C: FusedIterator<Item = Coordinate<T>>> Iterator for Close<T, C> {
    type Item = C::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.current {
            self.current = self.curve.next();
        }
        if let None = self.first {
            self.first = self.current.clone();
        }
        let mut ret = self.current.take();
        self.current = self.curve.next();
        if self.current.is_none() && self.first == ret {
            self.first.take();
        }
        if ret.is_none() && self.first.is_some() {
            std::mem::swap(&mut ret, &mut self.first);
        }
        return ret;
    }
}

#[test]
fn close_curve() {
    fn to_coordinate(a: f64) -> Coordinate<f64> {
        Coordinate::from([a, a + 1.])
    }
    let c1 = vec![1., 2., 3.];
    let it1 = c1.iter().map(|x| to_coordinate(*x));
    assert_eq!(
        it1.clone().into_curve().close().last(),
        to_coordinate(1.).into()
    );
    assert_eq!(it1.clone().into_curve().close().count(), c1.len() + 1);
    let c2 = vec![1., 1., 1.];
    let it2 = c2.iter().map(|x| to_coordinate(*x));
    assert_eq!(
        it2.clone().into_curve().close().last(),
        to_coordinate(1.).into()
    );
    assert_eq!(it2.clone().into_curve().close().count(), c2.len());
}

impl<T, C> Iterator for Curve<T, C>
where
    T: Scalar,
    C: Iterator<Item = Coordinate<T>>,
{
    type Item = Coordinate<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.curve.next()
    }
}

impl<T, C> DoubleEndedIterator for Curve<T, C>
where
    T: Scalar,
    C: DoubleEndedIterator<Item = Coordinate<T>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.curve.next_back()
    }
}

impl<T, C> ExactSizeIterator for Curve<T, C>
where
    T: Scalar,
    C: ExactSizeIterator<Item = Coordinate<T>>,
{
    fn len(&self) -> usize {
        self.curve.len()
    }
}

impl<T, C> FusedIterator for Curve<T, C>
where
    T: Scalar,
    C: FusedIterator<Item = Coordinate<T>>,
{
}

pub trait IntoCurve<T: Scalar> {
    type Curve: Iterator<Item = Coordinate<T>>;
    fn into_curve(self) -> Curve<T, Self::Curve>;
}

impl<T, C, S> IntoCurve<T> for C
where
    T: Scalar,
    C: IntoIterator<Item = S>,
    S: Into<Coordinate<T>>,
{
    type Curve = Map<C::IntoIter, fn(S) -> Coordinate<T>>;
    fn into_curve(self) -> Curve<T, Self::Curve> {
        Curve {
            curve: self.into_iter().map(|x| x.into()),
        }
    }
}

pub struct Area<T: Scalar, A: Iterator<Item = Coordinate<T>>> {
    area: A,
}

impl<T: Scalar, A: Iterator<Item = Coordinate<T>>> Area<T, A> {
    pub fn new(area: A) -> Self {
        Self { area }
    }
}

impl<T, A> Iterator for Area<T, A>
where
    T: Scalar,
    A: Iterator<Item = Coordinate<T>>,
{
    type Item = Coordinate<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.area.next()
    }
}

impl<T, A> DoubleEndedIterator for Area<T, A>
where
    T: Scalar,
    A: DoubleEndedIterator<Item = Coordinate<T>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.area.next_back()
    }
}

impl<T, A> ExactSizeIterator for Area<T, A>
where
    T: Scalar,
    A: ExactSizeIterator<Item = Coordinate<T>>,
{
    fn len(&self) -> usize {
        self.area.len()
    }
}

impl<T: Scalar, A> FusedIterator for Area<T, A> where A: FusedIterator<Item = Coordinate<T>> {}
pub trait Sweep<T: Scalar> {
    type Output: Iterator<Item = Coordinate<T>>;
    fn sweep(self, range: (T, T)) -> Area<T, Self::Output>;
}

pub trait Bias<T: Scalar>: IntoIterator<Item = Coordinate<T>> {
    fn bias(self, b: T) -> Self;
}

impl<C, T> Sweep<T> for C
where
    T: Scalar,
    C: Bias<T> + Clone,
    <C as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    type Output = Chain<C::IntoIter, Rev<C::IntoIter>>;
    fn sweep(self, range: (T, T)) -> Area<T, Self::Output> {
        Area::new(
            self.clone()
                .bias(range.0)
                .into_iter()
                .chain(self.bias(range.1).into_iter().rev()),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct _Arc<S> {
    radius: S,
}

impl<S> _Arc<S> {
    fn new(radius: S) -> Self {
        Self { radius }
    }
    fn to_points(
        self,
        angle: (Angle<<S as Distance>::Basic>, Angle<<S as Distance>::Basic>),
        resolution: Resolution<S>,
    ) -> impl DoubleEndedIterator<Item = Coordinate<S>>
    where
        S: Distance + Copy,
        <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
    {
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
pub struct CircularArc<S> {
    inner: _Arc<S>,
    center: (S, S),
    angle: (Angle<S>, Angle<S>),
}
