use crate::Quantity;

use self::groups::{Compound, Group};

use super::coordinate::Coordinate;

#[cfg(feature = "rayon")]
pub mod par_iter;

pub mod iter;
pub mod groups;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Curve<C> {
    curve: C,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Area<A> {
    area: A,
}

pub trait IntoCurve {
    type Q: Quantity;
    type Curve: IntoIterator<Item = Coordinate<Self::Q>>;
    fn into_curve(self) -> Curve<Self::Curve>;
}

pub trait IntoArea {
    type Q: Quantity;
    type Area: IntoIterator<Item = Coordinate<Self::Q>>;
    fn into_area(self) -> Area<Self::Area>;
}

impl<A, Q> IntoArea for Area<A>
where
    Q: Quantity,
    A: IntoIterator<Item = Coordinate<Q>>,
{
    type Q = Q;
    type Area = A;
    fn into_area(self) -> Area<Self::Area> {
        self
    }
}

pub trait Sweep<Q: Quantity> {
    type Output: IntoIterator<Item = Coordinate<Q>>;
    fn sweep(self, range: (Q, Q)) -> Area<Self::Output>;
}

pub trait Bias<Q: Quantity>: IntoIterator<Item = Coordinate<Q>> {
    fn bias(&mut self, b: Q) -> &mut Self;
}

pub trait Split<P>: Sized {
    fn split(self, pos: P) -> (Self, Self);
}

impl<C, Q> Bias<Q> for Curve<C>
where
    C: Bias<Q> + IntoIterator<Item = Coordinate<Q>>,
    Q: Quantity,
{
    fn bias(&mut self, b: Q) -> &mut Self {
        self.curve.bias(b);
        self
    }
}

impl<Q, T1, T2> Bias<Q> for Compound<T1, T2>
where
    Q: Quantity,
    T1: Bias<Q>,
    T2: Bias<Q>,
{
    fn bias(&mut self, b: Q) -> &mut Self {
        self.0.bias(b.clone());
        self.1.bias(b);
        self
    }
}

impl<Q, T> Bias<Q> for Group<T>
where
    Q: Quantity,
    T: Bias<Q>,
{
    fn bias(&mut self, b: Q) -> &mut Self {
        for t in self.0.iter_mut() {
            t.bias(b.clone());
        }
        self
    }
}

impl<P, T1, T2> Split<P> for Compound<T1, T2>
where
    P: Clone,
    T1: Split<P>,
    T2: Split<P>,
{
    fn split(self, pos: P) -> (Self, Self) {
        let t1 = self.0.split(pos.clone());
        let t2 = self.1.split(pos);
        (Self(t1.0, t2.0), Self(t1.1, t2.1))
    }
}

impl<P, T> Split<P> for Group<T>
where
    P: Clone,
    T: Split<P>,
{
    fn split(self, pos: P) -> (Self, Self) {
        let (left, right): (Vec<_>, Vec<_>) = self
            .0
            .into_iter()
            .map(move |x| x.split(pos.clone()))
            .unzip();
        (Group(left), Group(right))
    }
}
