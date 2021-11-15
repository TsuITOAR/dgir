use crate::{draw::coordinate::Coordinate, Quantity};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Compound<T1, T2>(pub(crate) T1, pub(crate) T2);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group<T>(pub(crate) Vec<T>);

pub trait IntoCompound<T1, T2> {
    fn into_compound(self) -> Compound<T1, T2>;
}

impl<T1, T2, F> IntoCompound<T1, T2> for F
where
    F: Into<Compound<T1, T2>>,
{
    fn into_compound(self) -> Compound<T1, T2> {
        self.into()
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

pub trait IntoGroup<T> {
    fn into_group(self) -> Group<T>;
}

impl<Q, T, const LEN: usize> IntoGroup<T> for [T; LEN]
where
    Q: Quantity,
    T: IntoIterator<Item = Coordinate<Q>>,
{
    fn into_group(self) -> Group<T> {
        Group(Vec::from(self))
    }
}

impl<Q, T> IntoGroup<T> for Vec<T>
where
    Q: Quantity,
    T: IntoIterator<Item = Coordinate<Q>>,
{
    fn into_group(self) -> Group<T> {
        Group(self)
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

pub trait GraphIterator<'a, Q: Quantity> {
    type GraphIter: Iterator<Item = Self::PointIter> + 'a;
    type PointIter: Iterator<Item = Coordinate<Q>> + 'a;
    fn unzip(self) -> Self::GraphIter;
}

impl<'a, T1, T2, Q> GraphIterator<'a, Q> for Compound<T1, T2>
where
    Q: Quantity,
    T1: Iterator<Item = Coordinate<Q>> + 'a,
    T2: Iterator<Item = Coordinate<Q>> + 'a,
{
    type GraphIter = <Vec<Self::PointIter> as IntoIterator>::IntoIter;
    type PointIter = Box<dyn Iterator<Item = Coordinate<Q>> + 'a>;
    fn unzip(self) -> Self::GraphIter {
        vec![
            Box::new(self.0) as Self::PointIter,
            Box::new(self.1) as Self::PointIter,
        ]
        .into_iter()
    }
}

impl<'a, T, Q> GraphIterator<'a, Q> for Group<T>
where
    Q: Quantity,
    T: Iterator<Item = Coordinate<Q>> + 'a,
{
    type GraphIter = <Vec<Self::PointIter> as IntoIterator>::IntoIter;
    type PointIter = T;
    fn unzip(self) -> Self::GraphIter {
        self.0.into_iter()
    }
}
