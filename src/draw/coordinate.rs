use std::{
    mem::{self, size_of},
    ops::{Add, Index, IndexMut, Mul, Sub},
};

use nalgebra::{Point2, Scalar};

use super::Distance;

pub(crate) struct Coordinate<T: Scalar>(Point2<T>);

impl<T: Distance> Coordinate<T> {
    fn to_basic(self) -> Coordinate<T::Basic> {
        unsafe {
            debug_assert_eq!(size_of::<T>(), size_of::<T::Basic>());
            let ret = ((&self as *const Coordinate<T>) as *const Coordinate<T::Basic>).read();
            mem::forget(self);
            ret
        }
    }
    fn from_basic(basic: Coordinate<T::Basic>) -> Self {
        unsafe {
            debug_assert_eq!(size_of::<T>(), size_of::<T::Basic>());
            let ret = ((&basic as *const Coordinate<T::Basic>) as *const Coordinate<T>).read();
            mem::forget(basic);
            ret
        }
    }
}

impl<F, T: Distance> From<F> for Coordinate<T>
where
    Point2<T>: From<F>,
{
    fn from(f: F) -> Self {
        Self(Point2::from(f))
    }
}

impl<I, T: Distance> Index<I> for Coordinate<T>
where
    Point2<T>: Index<I>,
{
    type Output = <Point2<T> as Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I, T: Distance> IndexMut<I> for Coordinate<T>
where
    Point2<T>: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<A, T: Distance> Add<A> for Coordinate<T>
where
    Point2<T>: Add<A, Output = Point2<T>>,
{
    type Output = Self;
    fn add(self, rhs: A) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl<S, T: Distance> Sub<S> for Coordinate<T>
where
    Point2<T>: Sub<S, Output = Point2<T>>,
{
    type Output = Self;
    fn sub(self, rhs: S) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl<M, T: Distance> Mul<Coordinate<T>> for Wrapper<M>
where
    M: Mul<Point2<T::Basic>, Output = Point2<T::Basic>>,
{
    type Output = Coordinate<T>;
    fn mul(self, rhs: Coordinate<T>) -> Self::Output {
        Coordinate::from_basic(Coordinate(self.0 * rhs.to_basic().0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Wrapper<M>(pub(crate) M);

impl<M> From<M> for Wrapper<M> {
    fn from(m: M) -> Self {
        Self(m)
    }
}
