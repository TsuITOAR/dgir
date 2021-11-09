use std::{
    fmt::Debug,
    mem,
    ops::{Add, Index, IndexMut, Mul, Sub},
};

use nalgebra::Point2;

use crate::{
    units::{Length, LengthType},
    Num, Quantity,
};

// #[cfg(algebra)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Coordinate<Q: Quantity>(pub(crate) Point2<Q>);

pub(crate) type LenCo<L, T> = Coordinate<Length<L, T>>;

impl<Q: Quantity> From<Coordinate<Q>> for [Q; 2] {
    fn from(c: Coordinate<Q>) -> Self {
        <[Q; 2]>::from(c.0.coords)
    }
}

#[test]
fn coordinate_to_array() {
    assert_eq!([1., 2.], <[_; 2]>::from(Coordinate::from([1., 2.])))
}

impl<L: LengthType, T: Num> LenCo<L, T> {
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

impl<F, Q: Quantity> From<F> for Coordinate<Q>
where
    Point2<Q>: From<F>,
{
    fn from(f: F) -> Self {
        Self(Point2::from(f))
    }
}

impl<I, Q: Quantity> Index<I> for Coordinate<Q>
where
    Point2<Q>: Index<I>,
{
    type Output = <Point2<Q> as Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I, Q: Quantity> IndexMut<I> for Coordinate<Q>
where
    Point2<Q>: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<A, Q: Quantity> Add<A> for Coordinate<Q>
where
    Point2<Q>: Add<A, Output = Point2<Q>>,
{
    type Output = Self;
    fn add(self, rhs: A) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl<S, Q: Quantity> Sub<S> for Coordinate<Q>
where
    Point2<Q>: Sub<S, Output = Point2<Q>>,
{
    type Output = Self;
    fn sub(self, rhs: S) -> Self::Output {
        Self(self.0 - rhs)
    }
}


#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MulAsScalar<M>(pub(crate) M);

impl<M> From<M> for MulAsScalar<M> {
    fn from(m: M) -> Self {
        Self(m)
    }
}

impl<M, L: LengthType, T: Num> Mul<LenCo<L, T>> for MulAsScalar<M>
where
    M: Mul<Point2<T>, Output = Point2<T>>,
{
    type Output = LenCo<L, T>;
    fn mul(self, rhs: LenCo<L, T>) -> Self::Output {
        Coordinate::from_basic(Coordinate(self.0 * rhs.to_basic().0))
    }
}