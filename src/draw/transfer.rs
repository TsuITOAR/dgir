use std::{iter::Map, ops::Mul};

use nalgebra::{ClosedAdd, RealField, Rotation2, Similarity, Translation};

use crate::{
    units::{Angle, Length, LengthType},
    Num, Quantity,
};

use super::{
    coordinate::Coordinate,
    coordinate::MulAsScalar,
    curve::{
        groups::{Compound, Group},
        Area, Curve,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MulOpClosure<M> {
    pub m: M,
}

impl<S, M> FnOnce<(S,)> for MulOpClosure<M>
where
    for<'a> &'a M: Mul<S, Output = S>,
{
    type Output = S;
    extern "rust-call" fn call_once(self, args: (S,)) -> Self::Output {
        &self.m * args.0
    }
}

impl<S, M> FnMut<(S,)> for MulOpClosure<M>
where
    for<'a> &'a M: Mul<S, Output = S>,
{
    extern "rust-call" fn call_mut(&mut self, args: (S,)) -> Self::Output {
        (&self.m) * args.0
    }
}

impl<S, M> Fn<(S,)> for MulOpClosure<M>
where
    for<'a> &'a M: Mul<S, Output = S>,
{
    extern "rust-call" fn call(&self, args: (S,)) -> Self::Output {
        (&self.m) * args.0
    }
}

pub trait Transfer<Q>: Sized
where
    Q: Quantity,
{
    type Output<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone>: Transfer<Q>;
    fn transfer<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone>(self, f: F) -> Self::Output<F>;
    fn matrix_trans<M: Clone>(self, m: M) -> Self::Output<MulOpClosure<M>>
    where
        for<'a> &'a M: Mul<Coordinate<Q>, Output = Coordinate<Q>> + Copy,
    {
        self.transfer(MulOpClosure { m })
    }
}

pub trait CommonTrans<L, T>: Transfer<Length<L, T>>
where
    L: LengthType,
    T: Num,
{
    fn translate(
        self,
        x: Length<L, T>,
        y: Length<L, T>,
    ) -> Self::Output<MulOpClosure<MulAsScalar<Translation<T, 2>>>>
    where
        T: ClosedAdd + Copy,
    {
        self.matrix_trans(MulAsScalar(Translation::<T, 2>::from([x.value, y.value])))
    }
    fn scale(
        self,
        scale: T,
    ) -> Self::Output<MulOpClosure<MulAsScalar<Similarity<T, Rotation2<T>, 2>>>>
    where
        T: RealField + Copy,
    {
        self.matrix_trans(MulAsScalar(Similarity::<T, Rotation2<T>, 2>::from_scaling(
            scale,
        )))
    }
    fn rotate(self, ang: Angle<T>) -> Self::Output<MulOpClosure<MulAsScalar<Rotation2<T>>>>
    where
        T: RealField + Copy,
    {
        self.matrix_trans(MulAsScalar(Rotation2::new(ang.to_rad())))
    }
}

impl<L, T, U> CommonTrans<L, T> for U
where
    L: LengthType,
    T: Num,
    U: Transfer<Length<L, T>>,
{
}

impl<Q: Quantity, C: IntoIterator<Item = Coordinate<Q>>> Transfer<Q> for Curve<C> {
    type Output<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone> = Curve<Map<C::IntoIter, F>>;
    fn transfer<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone>(self, f: F) -> Self::Output<F> {
        Curve {
            curve: self.curve.into_iter().map(f),
        }
    }
}

impl<Q: Quantity, A: IntoIterator<Item = Coordinate<Q>>> Transfer<Q> for Area<A> {
    type Output<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone> = Area<Map<A::IntoIter, F>>;
    fn transfer<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone>(self, f: F) -> Self::Output<F> {
        Area {
            area: self.area.into_iter().map(f),
        }
    }
}

impl<Q, T1, T2> Transfer<Q> for Compound<T1, T2>
where
    Q: Quantity,
    T1: Transfer<Q>,
    T2: Transfer<Q>,
{
    type Output<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone> =
        Compound<T1::Output<F>, T2::Output<F>>;
    fn transfer<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone>(self, f: F) -> Self::Output<F> {
        Compound::from((self.0.transfer(f.clone()), self.1.transfer(f.clone())))
    }
}

impl<Q, T> Transfer<Q> for Group<T>
where
    Q: Quantity,
    T: Transfer<Q>,
{
    type Output<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone> = Group<T::Output<F>>;
    fn transfer<F: FnMut(Coordinate<Q>) -> Coordinate<Q> + Clone>(self, f: F) -> Self::Output<F> {
        Group(self.0.into_iter().map(|x| x.transfer(f.clone())).collect())
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::ApproxEq;

    use crate::{
        draw::{curve::IntoCurve, APPROX_EQ_MARGIN},
        MILLIMETER,
    };

    use super::*;
    #[test]
    fn trans_coordinate() {
        let coor = Coordinate::from((MILLIMETER, MILLIMETER * 2.));
        assert_eq!(
            std::iter::once(coor)
                .into_curve()
                .scale(2.)
                .into_iter()
                .next()
                .unwrap(),
            Coordinate::from((MILLIMETER * 2., MILLIMETER * 4.))
        );
        assert_eq!(
            std::iter::once(coor)
                .into_curve()
                .translate(MILLIMETER, MILLIMETER * -1.)
                .into_iter()
                .next()
                .unwrap(),
            Coordinate::from((MILLIMETER * 2., MILLIMETER))
        );
        assert!(std::iter::once(coor)
            .into_curve()
            .rotate(Angle::from_deg(90.))
            .into_iter()
            .next()
            .unwrap()
            .approx_eq(
                Coordinate::from((MILLIMETER * -2., MILLIMETER)),
                APPROX_EQ_MARGIN
            ),);
    }
}
