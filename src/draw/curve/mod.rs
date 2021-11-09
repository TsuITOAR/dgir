use crate::Quantity;

use super::coordinate::Coordinate;

#[cfg(feature = "rayon")]
pub mod par_iter;

pub mod iter;

pub struct Curve<C> {
    curve: C,
}

pub struct Area<A> {
    area: A,
}

pub trait Sweep<T: Quantity> {
    type Output: Iterator<Item = Coordinate<T>>;
    fn sweep(self, range: (T, T)) -> Area<Self::Output>;
}

pub trait Bias<T: Quantity>: IntoIterator<Item = Coordinate<T>> {
    fn bias(self, b: T) -> Self;
}
