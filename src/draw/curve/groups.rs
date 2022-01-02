use crate::{draw::coordinate::Coordinate, Quantity};

use super::Area;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Compound<T1, T2>(pub(crate) T1, pub(crate) T2);

impl<Q, T1, T2> Compound<T1, T2>
where
    T1: IntoIterator<Item = Coordinate<Q>>,
    T2: IntoIterator<Item = Coordinate<Q>>,
    Q: Quantity,
{
    pub fn fusion(self) -> impl Iterator<Item = Coordinate<Q>> {
        self.into_iter()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group<T>(pub(crate) Vec<T>);

impl<Q, T> Group<T>
where
    T: IntoIterator<Item = Coordinate<Q>>,
    Q: Quantity,
{
    pub fn fusion(self) -> impl Iterator<Item = Coordinate<Q>> {
        self.into_iter()
    }
}

impl<T1, T2, Q> From<(T1, T2)> for Compound<T1, T2>
where
    Q: Quantity,
    T1: IntoIterator<Item = Coordinate<Q>>,
    T2: IntoIterator<Item = Coordinate<Q>>,
{
    fn from(f: (T1, T2)) -> Self {
        Self(f.0, f.1)
    }
}

impl<Q, T, const LEN: usize> From<[T; LEN]> for Group<T>
where
    Q: Quantity,
    T: IntoIterator<Item = Coordinate<Q>>,
{
    fn from(f: [T; LEN]) -> Self {
        Self(Vec::from(f))
    }
}

impl<Q, T> From<Vec<T>> for Group<T>
where
    Q: Quantity,
    T: IntoIterator<Item = Coordinate<Q>>,
{
    fn from(f: Vec<T>) -> Self {
        Self(f)
    }
}

impl<Q, T> From<(T, T)> for Group<T>
where
    Q: Quantity,
    T: IntoIterator<Item = Coordinate<Q>>,
{
    fn from(f: (T, T)) -> Self {
        Self(Vec::from([f.0, f.1]))
    }
}

impl<T1, T2, Q> IntoIterator for Compound<T1, T2>
where
    Q: Quantity,
    T1: IntoIterator<Item = Coordinate<Q>>,
    T2: IntoIterator<Item = Coordinate<Q>>,
{
    type IntoIter = std::iter::Chain<T1::IntoIter, T2::IntoIter>;
    type Item = Coordinate<Q>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().chain(self.1.into_iter())
    }
}

impl<Q, T> IntoIterator for Group<T>
where
    Q: Quantity,
    T: IntoIterator<Item = Coordinate<Q>>,
{
    type IntoIter = std::iter::Flatten<<Vec<T> as IntoIterator>::IntoIter>;
    type Item = Coordinate<Q>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().flatten()
    }
}

pub trait GraphIterator<'a, Q: Quantity> {
    type GraphIter: Iterator<Item = Self::PointIter> + 'a;
    type PointIter: Iterator<Item = Coordinate<Q>> + 'a;
    fn unzip(self) -> Self::GraphIter;
}

impl<'a, Q, A> GraphIterator<'a, Q> for Area<A>
where
    Q: Quantity,
    Area<A>: IntoIterator<Item = Coordinate<Q>> + 'a,
{
    type GraphIter = std::iter::Once<<Self as IntoIterator>::IntoIter>;
    type PointIter = <Self as IntoIterator>::IntoIter;
    fn unzip(self) -> Self::GraphIter {
        std::iter::once(self.into_iter())
    }
}

impl<'a, T1, T2, Q> GraphIterator<'a, Q> for Compound<T1, T2>
where
    Q: Quantity,
    T1: GraphIterator<'a, Q>,
    T2: GraphIterator<'a, Q>,
{
    type GraphIter = std::iter::Chain<
        std::iter::Map<
            <T1 as GraphIterator<'a, Q>>::GraphIter,
            fn(<T1 as GraphIterator<'a, Q>>::PointIter) -> Self::PointIter,
        >,
        std::iter::Map<
            <T2 as GraphIterator<'a, Q>>::GraphIter,
            fn(<T2 as GraphIterator<'a, Q>>::PointIter) -> Self::PointIter,
        >,
    >;
    type PointIter = Box<dyn Iterator<Item = Coordinate<Q>> + 'a>;
    fn unzip(self) -> Self::GraphIter {
        self.0
            .unzip()
            .map((|i| Box::new(i) as Self::PointIter) as fn(_) -> _)
            .chain(
                self.1
                    .unzip()
                    .map((|i| Box::new(i) as Self::PointIter) as fn(_) -> _),
            )
    }
}

impl<'a, T, Q> GraphIterator<'a, Q> for Group<T>
where
    Q: Quantity,
    T: GraphIterator<'a, Q> + 'a,
{
    type GraphIter =
        std::iter::Flatten<std::iter::Map<std::vec::IntoIter<T>, fn(T) -> T::GraphIter>>;
    type PointIter = T::PointIter;
    fn unzip(self) -> Self::GraphIter {
        self.0
            .into_iter()
            .map((|x: T| x.unzip()) as fn(_) -> _)
            .flatten()
    }
}
