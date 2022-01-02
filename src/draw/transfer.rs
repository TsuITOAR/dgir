use std::{
    iter::{FusedIterator, Map},
    ops::Mul,
};

use nalgebra::{ClosedAdd, RealField, Rotation2, Similarity, Translation};

use crate::{
    units::{Angle, Length, LengthType},
    Num, Quantity,
};

use super::{
    coordinate::Coordinate,
    coordinate::{LenCo, MulAsScalar},
};

pub trait IntoTransfer<Q: Quantity, S: Iterator<Item = Coordinate<Q>>> {
    fn into_transfer(self) -> Transfer<Q, S>;
}

impl<Q, S> IntoTransfer<Q, S> for S
where
    Q: Quantity,
    S: Iterator<Item = Coordinate<Q>>,
{
    fn into_transfer(self) -> Transfer<Q, S> {
        Transfer { s: self }
    }
}

pub struct Transfer<Q: Quantity, S: Iterator<Item = Coordinate<Q>>> {
    s: S,
}

impl<Q, S> Iterator for Transfer<Q, S>
where
    Q: Quantity,
    S: Iterator<Item = Coordinate<Q>>,
{
    type Item = Coordinate<Q>;
    fn next(&mut self) -> Option<Self::Item> {
        self.s.next()
    }
}

impl<Q, S> DoubleEndedIterator for Transfer<Q, S>
where
    Q: Quantity,
    S: DoubleEndedIterator<Item = Coordinate<Q>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.s.next_back()
    }
}

impl<Q, S> ExactSizeIterator for Transfer<Q, S>
where
    Q: Quantity,
    S: ExactSizeIterator<Item = Coordinate<Q>>,
{
    fn len(&self) -> usize {
        self.s.len()
    }
}

impl<Q: Quantity, S> FusedIterator for Transfer<Q, S> where S: FusedIterator<Item = Coordinate<Q>> {}

impl<Q: Quantity, S: Iterator<Item = Coordinate<Q>>> Transfer<Q, S> {
    pub fn new(s: S) -> Self {
        Self { s }
    }
    pub fn into_inner(self) -> S {
        self.s
    }
    pub fn transfer<F: FnMut(Coordinate<Q>) -> Coordinate<Q>>(
        self,
        f: F,
    ) -> Transfer<Q, Map<S, F>> {
        Transfer::new(self.s.map(f))
    }
    pub fn matrix_trans<M>(
        self,
        m: M,
    ) -> Transfer<Q, Map<S, impl FnMut(Coordinate<Q>) -> Coordinate<Q>>>
    where
        M: Mul<Coordinate<Q>, Output = Coordinate<Q>> + Copy,
    {
        self.transfer(move |s: Coordinate<Q>| -> Coordinate<Q> { m * s })
    }
}

impl<L, T, S> Transfer<Length<L, T>, S>
where
    L: LengthType,
    T: Num,
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
        self.matrix_trans(MulAsScalar(Translation::<T, 2>::from([x.value, y.value])))
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

#[cfg(test)]
mod tests {
    use float_cmp::ApproxEq;

    use crate::{draw::APROX_EQ_MARGIN, MILLIMETER};

    use super::*;
    #[test]
    fn trans_coordinate() {
        let coor = Coordinate::from((MILLIMETER, MILLIMETER * 2.));
        assert_eq!(
            std::iter::once(coor)
                .into_transfer()
                .scale(2.)
                .next()
                .unwrap(),
            Coordinate::from((MILLIMETER * 2., MILLIMETER * 4.))
        );
        assert_eq!(
            std::iter::once(coor)
                .into_transfer()
                .translate(MILLIMETER, MILLIMETER * -1.)
                .next()
                .unwrap(),
            Coordinate::from((MILLIMETER * 2., MILLIMETER))
        );
        assert!(std::iter::once(coor)
            .into_transfer()
            .rotate(Angle::from_deg(90.))
            .next()
            .unwrap()
            .approx_eq(
                Coordinate::from((MILLIMETER * -2., MILLIMETER)),
                APROX_EQ_MARGIN
            ),);
    }
}
