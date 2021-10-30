use std::{
    iter::{FusedIterator, Map},
    ops::Mul,
};

use nalgebra::{ClosedAdd, RealField, Rotation2, Scalar, Similarity, Translation2};

use crate::units::Angle;

use super::Coordinate;

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
    pub fn translate(
        self,
        x: T,
        y: T,
    ) -> Transfer<T, Map<S, impl FnMut(Coordinate<T>) -> Coordinate<T>>>
    where
        T: ClosedAdd + Copy,
    {
        self.matrix_trans(Translation2::new(x, y))
    }
    pub fn scale<'a, X: Into<Option<(T, T)>>>(
        self,
        scale: T,
    ) -> Transfer<T, Map<S, impl FnMut(Coordinate<T>) -> Coordinate<T>>>
    where
        T: RealField + Copy,
    {
        self.matrix_trans(Similarity::<T, Rotation2<T>, 2>::from_scaling(scale))
    }
    pub fn rotate(
        self,
        ang: Angle<T>,
    ) -> Transfer<T, Map<S, impl FnMut(Coordinate<T>) -> Coordinate<T>>>
    where
        T: RealField + Copy,
    {
        self.matrix_trans(Rotation2::new(ang.to_rad()))
    }
}
