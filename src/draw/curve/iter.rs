use std::iter::{Fuse, FusedIterator, Rev};

use crate::Num;

use crate::{
    color::LayerData,
    draw::coordinate::Coordinate,
    gds::{Element, Path, Polygon},
    units::{Length, LengthType},
    Quantity,
};

use super::groups::Compound;
use super::{Area, Bias, Curve, IntoArea, IntoCurve, Sweep};

impl<T: Num, C: Iterator<Item = Coordinate<T>>> Curve<C> {
    pub fn new(curve: C) -> Self {
        Self { curve }
    }
    pub fn close(self) -> Area<Close<T, Fuse<C>>>
    where
        T: Copy + PartialEq,
        C: Iterator<Item = Coordinate<T>>,
    {
        Close {
            curve: self.curve.fuse(),
            first: None,
            current: None,
        }
        .into_area()
    }
}

impl<L, T, C> Curve<C>
where
    L: LengthType,
    T: Num,
    C: Iterator<Item = Coordinate<Length<L, T>>> + 'static,
{
    pub fn to_path(self, color: LayerData) -> Element<L, T> {
        Path {
            curve: Box::new(self.curve),
            color,
            width: None,
        }
        .into()
    }
    pub fn width_path(self, width: Length<L, T>, color: LayerData) -> Element<L, T> {
        Path {
            curve: Box::new(self.curve),
            color,
            width: Some(width),
        }
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct Close<T: Quantity, C: FusedIterator<Item = Coordinate<T>>> {
    curve: C,
    first: Option<C::Item>,
    current: Option<C::Item>,
}

impl<T: Quantity, C: FusedIterator<Item = Coordinate<T>>> Iterator for Close<T, C> {
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

impl<Q, C> IntoArea for Close<Q, C>
where
    Q: Quantity,
    C: FusedIterator<Item = Coordinate<Q>>,
{
    type Q = Q;
    type Area = Self;
    fn into_area(self) -> Area<Self::Area> {
        Area { area: self }
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
        it1.clone().into_curve().close().into_iter().last(),
        to_coordinate(1.).into()
    );
    assert_eq!(
        it1.clone().into_curve().close().into_iter().count(),
        c1.len() + 1
    );
    let c2 = vec![1., 1., 1.];
    let it2 = c2.iter().map(|x| to_coordinate(*x));
    assert_eq!(
        it2.clone().into_curve().close().into_iter().last(),
        to_coordinate(1.).into()
    );
    assert_eq!(
        it2.clone().into_curve().close().into_iter().count(),
        c2.len()
    );
}

impl<Q, C> IntoIterator for Curve<C>
where
    Q: Quantity,
    C: IntoIterator<Item = Coordinate<Q>>,
{
    type IntoIter = C::IntoIter;
    type Item = Coordinate<Q>;
    fn into_iter(self) -> Self::IntoIter {
        self.curve.into_iter()
    }
}

impl<Q, C> IntoCurve for C
where
    Q: Quantity,
    C: IntoIterator<Item = Coordinate<Q>>,
{
    type Q = Q;
    type Curve = C::IntoIter;
    fn into_curve(self) -> Curve<Self::Curve> {
        Curve {
            curve: self.into_iter(),
        }
    }
}

impl<L, T, A> Area<A>
where
    L: LengthType,
    T: Num,
    A: Iterator<Item = Coordinate<Length<L, T>>> + 'static,
{
    pub fn to_polygon(self, color: LayerData) -> Element<L, T> {
        Polygon {
            area: Box::new(self.area),
            color,
        }
        .into()
    }
}

impl<Q, A> IntoIterator for Area<A>
where
    Q: Quantity,
    A: IntoIterator<Item = Coordinate<Q>>,
{
    type IntoIter = A::IntoIter;
    type Item = Coordinate<Q>;
    fn into_iter(self) -> Self::IntoIter {
        self.area.into_iter()
    }
}

impl<C, Q> Sweep<Q> for C
where
    Q: Quantity,
    C: Bias<Q> + Clone,
    <C as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    type Output = Compound<C::IntoIter, Rev<C::IntoIter>>;
    fn sweep(self, range: (Q, Q)) -> Area<Self::Output> {
        let mut t1 = self.clone();
        let mut t2 = self;
        t1.bias(range.0);
        t2.bias(range.1);
        Area {
            area: Compound::from((t1.into_iter(), t2.into_iter().rev())),
        }
    }
}
