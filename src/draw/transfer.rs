use std::{
    iter::{FusedIterator, Map},
    ops::Mul,
};

use nalgebra::{ClosedAdd, RealField, Rotation2, Scalar, Similarity, Translation};
use num::Num;

use crate::units::{Angle, Length, LengthType};

use super::{
    coordinate::Coordinate,
    coordinate::{LenCo, MulAsScalar},
};

pub trait IntoTransfer<T: Scalar, S: Iterator<Item = Coordinate<T>>> {
    fn into_transfer(self) -> Transfer<T, S>;
}

pub struct Transfer<T: Scalar, S: Iterator<Item = Coordinate<T>>> {
    s: S,
}

impl<T, S> Iterator for Transfer<T, S>
where
    T: Scalar,
    S: Iterator<Item = Coordinate<T>>,
{
    type Item = Coordinate<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.s.next()
    }
}

impl<T, S> DoubleEndedIterator for Transfer<T, S>
where
    T: Scalar,
    S: DoubleEndedIterator<Item = Coordinate<T>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.s.next_back()
    }
}

impl<T, S> ExactSizeIterator for Transfer<T, S>
where
    T: Scalar,
    S: ExactSizeIterator<Item = Coordinate<T>>,
{
    fn len(&self) -> usize {
        self.s.len()
    }
}

impl<T: Scalar, S> FusedIterator for Transfer<T, S> where S: FusedIterator<Item = Coordinate<T>> {}

impl<T: Scalar, S: Iterator<Item = Coordinate<T>>> Transfer<T, S> {
    pub fn new(s: S) -> Self {
        Self { s }
    }
    pub fn into_inner(self) -> S {
        self.s
    }
    pub fn transfer<F: FnMut(Coordinate<T>) -> Coordinate<T>>(
        self,
        f: F,
    ) -> Transfer<T, Map<S, F>> {
        Transfer::new(self.s.map(f))
    }
    pub fn matrix_trans<M>(
        self,
        m: M,
    ) -> Transfer<T, Map<S, impl FnMut(Coordinate<T>) -> Coordinate<T>>>
    where
        M: Mul<Coordinate<T>, Output = Coordinate<T>> + Copy,
    {
        self.transfer(move |s: Coordinate<T>| -> Coordinate<T> { m * s })
    }
}

impl<L, T, S> Transfer<Length<L, T>, S>
where
    L: LengthType,
    T: Scalar + Num,
    S: Iterator<Item = LenCo<L, T>>,
{
    pub fn translate(
        self,
        x: Length<L, T>,
        y: Length<L, T>,
    ) -> Transfer<Length<L, T>, Map<S, impl FnMut(LenCo<L, T>) -> LenCo<L, T>>>
    where
        T: ClosedAdd + Copy,
    {
        self.matrix_trans(MulAsScalar(Translation::<T, 2>::new(x.value, y.value)))
    }
    pub fn scale(
        self,
        scale: T,
    ) -> Transfer<Length<L, T>, Map<S, impl FnMut(LenCo<L, T>) -> LenCo<L, T>>>
    where
        T: RealField + Copy,
    {
        self.matrix_trans(MulAsScalar(Similarity::<T, Rotation2<T>, 2>::from_scaling(
            scale,
        )))
    }
    pub fn rotate(
        self,
        ang: Angle<T>,
    ) -> Transfer<Length<L, T>, Map<S, impl FnMut(LenCo<L, T>) -> LenCo<L, T>>>
    where
        T: RealField + Copy,
    {
        self.matrix_trans(MulAsScalar(Rotation2::new(ang.to_rad())))
    }
}
