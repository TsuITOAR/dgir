use std::iter::{Chain, Fuse, FusedIterator, Map, Rev};

use crate::Num;

use crate::{
    color::LayerData,
    draw::coordinate::Coordinate,
    gds::{Element, Path, Polygon, ToDgirElement},
    units::{Length, LengthType},
    Quantity,
};

use super::{Area, Bias, Curve, Sweep};

impl<T: Num, C: Iterator<Item = Coordinate<T>>> Curve<C> {
    pub fn new(curve: C) -> Self {
        Self { curve }
    }
    pub fn close(self) -> Area<Close<T, Fuse<C>>>
    where
        T: Copy + PartialEq,
        C: Iterator<Item = Coordinate<T>>,
    {
        Area::new(Close {
            curve: self.curve.fuse(),
            first: None,
            current: None,
        })
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
        .to_dgir_element()
    }
    pub fn width_path(self, width: Length<L, T>, color: LayerData) -> Element<L, T> {
        Path {
            curve: Box::new(self.curve),
            color,
            width: Some(width),
        }
        .to_dgir_element()
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

impl<Q, C> Iterator for Curve<C>
where
    Q: Quantity,
    C: Iterator<Item = Coordinate<Q>>,
{
    type Item = Coordinate<Q>;
    fn next(&mut self) -> Option<Self::Item> {
        self.curve.next()
    }
}

impl<Q, C> DoubleEndedIterator for Curve<C>
where
    Q: Quantity,
    C: DoubleEndedIterator<Item = Coordinate<Q>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.curve.next_back()
    }
}

impl<Q, C> ExactSizeIterator for Curve<C>
where
    Q: Quantity,
    C: ExactSizeIterator<Item = Coordinate<Q>>,
{
    fn len(&self) -> usize {
        self.curve.len()
    }
}

impl<Q, C> FusedIterator for Curve<C>
where
    Q: Quantity,
    C: FusedIterator<Item = Coordinate<Q>>,
{
}

pub trait IntoCurve<T: Quantity> {
    type Curve: Iterator<Item = Coordinate<T>>;
    fn into_curve(self) -> Curve<Self::Curve>;
}

impl<Q, C, S> IntoCurve<Q> for C
where
    Q: Quantity,
    C: IntoIterator<Item = S>,
    S: Into<Coordinate<Q>>,
{
    type Curve = Map<C::IntoIter, fn(S) -> Coordinate<Q>>;
    fn into_curve(self) -> Curve<Self::Curve> {
        Curve {
            curve: self.into_iter().map(|x| x.into()),
        }
    }
}

impl<Q: Quantity, A: Iterator<Item = Coordinate<Q>>> Area<A> {
    pub fn new(area: A) -> Self {
        Self { area }
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
        .to_dgir_element()
    }
}

impl<Q, A> Iterator for Area<A>
where
    Q: Quantity,
    A: Iterator<Item = Coordinate<Q>>,
{
    type Item = Coordinate<Q>;
    fn next(&mut self) -> Option<Self::Item> {
        self.area.next()
    }
}

impl<Q, A> DoubleEndedIterator for Area<A>
where
    Q: Quantity,
    A: DoubleEndedIterator<Item = Coordinate<Q>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.area.next_back()
    }
}

impl<Q, A> ExactSizeIterator for Area<A>
where
    Q: Quantity,
    A: ExactSizeIterator<Item = Coordinate<Q>>,
{
    fn len(&self) -> usize {
        self.area.len()
    }
}

impl<Q: Quantity, A> FusedIterator for Area<A> where A: FusedIterator<Item = Coordinate<Q>> {}

impl<C, Q> Sweep<Q> for C
where
    Q: Quantity,
    C: Bias<Q> + Clone,
    <C as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    type Output = Chain<C::IntoIter, Rev<C::IntoIter>>;
    fn sweep(self, range: (Q, Q)) -> Area<Self::Output> {
        Area::new(
            self.clone()
                .bias(range.0)
                .into_iter()
                .chain(self.bias(range.1).into_iter().rev()),
        )
    }
}
