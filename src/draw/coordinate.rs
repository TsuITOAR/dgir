use std::{
    mem,
    ops::{Add, Index, IndexMut, Mul, Sub},
};

use nalgebra::{Point2, Scalar};
use num::Num;

use crate::units::{Length, LengthType};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Coordinate<T: Scalar>(pub(crate) Point2<T>);

pub(crate) type LenCo<L, T> = Coordinate<Length<L, T>>;

impl<T:Scalar> Coordinate<T>{
    
}

impl<L: LengthType, T: Scalar + Num> LenCo<L, T> {
    fn to_basic(self) -> Coordinate<T> {
        unsafe {
            debug_assert_eq!(
                mem::size_of::<Coordinate<T>>(),
                mem::size_of::<LenCo<L, T>>()
            );
            let ret = ((&self as *const LenCo<L, T>) as *const Coordinate<T>).read();
            mem::forget(self);
            ret
        }
    }
    fn from_basic(basic: Coordinate<T>) -> Self {
        unsafe {
            debug_assert_eq!(
                mem::size_of::<Coordinate<T>>(),
                mem::size_of::<LenCo<L, T>>()
            );
            let ret = ((&basic as *const Coordinate<T>) as *const LenCo<L, T>).read();
            mem::forget(basic);
            ret
        }
    }
}

impl<F, T: Scalar> From<F> for Coordinate<T>
where
    Point2<T>: From<F>,
{
    fn from(f: F) -> Self {
        Self(Point2::from(f))
    }
}

impl<I, T: Scalar> Index<I> for Coordinate<T>
where
    Point2<T>: Index<I>,
{
    type Output = <Point2<T> as Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I, T: Scalar> IndexMut<I> for Coordinate<T>
where
    Point2<T>: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<A, T: Scalar> Add<A> for Coordinate<T>
where
    Point2<T>: Add<A, Output = Point2<T>>,
{
    type Output = Self;
    fn add(self, rhs: A) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl<S, T: Scalar> Sub<S> for Coordinate<T>
where
    Point2<T>: Sub<S, Output = Point2<T>>,
{
    type Output = Self;
    fn sub(self, rhs: S) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl<M, L: LengthType, T: Scalar + Num> Mul<LenCo<L, T>> for MulAsScalar<M>
where
    M: Mul<Point2<T>, Output = Point2<T>>,
{
    type Output = LenCo<L, T>;
    fn mul(self, rhs: LenCo<L, T>) -> Self::Output {
        Coordinate::from_basic(Coordinate(self.0 * rhs.to_basic().0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MulAsScalar<M>(pub(crate) M);

impl<M> From<M> for MulAsScalar<M> {
    fn from(m: M) -> Self {
        Self(m)
    }
}
