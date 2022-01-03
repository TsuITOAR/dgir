use nalgebra::RealField;
use num::{traits::FloatConst, Float, FromPrimitive};

use crate::{
    draw::{
        coordinate::{Coordinate, LenCo},
        curve::{
            groups::{Compound, Group},
            Area, Curve, Sweep,
        },
        transfer::CommonTrans,
        CircularArc, Line,
    },
    units::{Absolute, Angle, Length, LengthType},
    Num, Quantity,
};

pub struct Cursor<L = Absolute, T = f64>
where
    L: LengthType,
    T: Num,
{
    pos: LenCo<L, T>,
    dir: Angle<T>,
}

impl<L: LengthType, T: Num> Cursor<L, T> {
    pub fn new<C: Into<LenCo<L, T>>>(pos: C, dir: Angle<T>) -> Self {
        Self {
            pos: pos.into(),
            dir,
        }
    }
    pub fn assemble<E>(&mut self, e: E) -> impl IntoIterator<Item = LenCo<L, T>>
    where
        T: RealField,
        E: CommonTrans<L, T> + Pos<Length<L, T>> + Dir<T>,
    {
        let start_pos = e.start_pos();
        let end_pos = e.end_pos();
        let start_ang = e.start_ang();
        let end_ang = e.end_ang();
        let e = e
            .translate(-start_pos[0], -start_pos[1])
            .rotate(self.dir - start_ang)
            .translate(self.pos[0], self.pos[1]);
        self.pos = (self.pos + end_pos - start_pos).into();
        self.dir = self.dir + end_ang - start_ang;
        e
    }
}

pub trait Pos<Q: Quantity> {
    fn start_pos(&self) -> Coordinate<Q>;
    fn end_pos(&self) -> Coordinate<Q>;
}

pub trait Dir<A: Num> {
    fn start_ang(&self) -> Angle<A>;
    fn end_ang(&self) -> Angle<A>;
}

impl<Q: Quantity, C: Pos<Q>> Pos<Q> for Curve<C> {
    fn start_pos(&self) -> Coordinate<Q> {
        self.curve.start_pos()
    }
    fn end_pos(&self) -> Coordinate<Q> {
        self.curve.end_pos()
    }
}

impl<Q: Quantity, A: Pos<Q>> Pos<Q> for Area<A> {
    fn start_pos(&self) -> Coordinate<Q> {
        self.area.start_pos()
    }
    fn end_pos(&self) -> Coordinate<Q> {
        self.area.end_pos()
    }
}

impl<Q: Num, C: Dir<Q>> Dir<Q> for Curve<C> {
    fn start_ang(&self) -> Angle<Q> {
        self.curve.start_ang()
    }
    fn end_ang(&self) -> Angle<Q> {
        self.curve.end_ang()
    }
}

impl<Q: Num, A: Dir<Q>> Dir<Q> for Area<A> {
    fn start_ang(&self) -> Angle<Q> {
        self.area.start_ang()
    }
    fn end_ang(&self) -> Angle<Q> {
        self.area.end_ang()
    }
}

impl<L: LengthType, T: Num + Float> Pos<Length<L, T>> for CircularArc<L, T> {
    fn start_pos(&self) -> Coordinate<Length<L, T>> {
        (
            self.inner.radius * self.angle.0.to_rad().cos() + self.center.0,
            self.inner.radius * self.angle.0.to_rad().sin() + self.center.1,
        )
            .into()
    }
    fn end_pos(&self) -> Coordinate<Length<L, T>> {
        (
            self.inner.radius * self.angle.1.to_rad().cos() + self.center.0,
            self.inner.radius * self.angle.1.to_rad().sin() + self.center.1,
        )
            .into()
    }
}

impl<L: LengthType, T: Num + FromPrimitive> Dir<T> for CircularArc<L, T> {
    fn start_ang(&self) -> Angle<T> {
        if self.angle.0 < self.angle.1 {
            self.angle.0 + Angle::from_rad(T::from_f64(std::f64::consts::FRAC_PI_2).unwrap())
        } else {
            self.angle.0 - Angle::from_rad(T::from_f64(std::f64::consts::FRAC_PI_2).unwrap())
        }
    }
    fn end_ang(&self) -> Angle<T> {
        if self.angle.0 < self.angle.1 {
            self.angle.1 + Angle::from_rad(T::from_f64(std::f64::consts::FRAC_PI_2).unwrap())
        } else {
            self.angle.1 - Angle::from_rad(T::from_f64(std::f64::consts::FRAC_PI_2).unwrap())
        }
    }
}

impl<L: LengthType, T: Num + Float> Pos<Length<L, T>> for Line<L, T> {
    fn start_pos(&self) -> Coordinate<Length<L, T>> {
        self.start
    }
    fn end_pos(&self) -> Coordinate<Length<L, T>> {
        self.end
    }
}

impl<L: LengthType, T: Num + Float + FloatConst> Dir<T> for Line<L, T> {
    fn start_ang(&self) -> Angle<T> {
        let (x, y) = (self.end - self.start).into();
        if x.value.is_zero() {
            Angle::from_rad(T::FRAC_PI_2() * y.value.signum())
        } else {
            Angle::from_rad(
                (y / x).atan()
                    + if x.value.is_positive() {
                        T::zero()
                    } else {
                        T::PI()
                    },
            )
        }
    }
    fn end_ang(&self) -> Angle<T> {
        self.start_ang()
    }
}

impl<Q: Quantity, T1, T2> Pos<Q> for Compound<T1, T2>
where
    T1: Pos<Q>,
{
    fn start_pos(&self) -> Coordinate<Q> {
        self.0.start_pos()
    }
    fn end_pos(&self) -> Coordinate<Q> {
        self.0.end_pos()
    }
}

impl<Q: Quantity, T> Pos<Q> for Group<T>
where
    T: Pos<Q>,
{
    fn start_pos(&self) -> Coordinate<Q> {
        self.0
            .first()
            .map(|x| x.start_pos())
            .unwrap_or(Coordinate::from([Q::zero(), Q::zero()]))
    }
    fn end_pos(&self) -> Coordinate<Q> {
        self.0
            .first()
            .map(|x| x.end_pos())
            .unwrap_or(Coordinate::from([Q::zero(), Q::zero()]))
    }
}

impl<Q: Num, T1, T2> Dir<Q> for Compound<T1, T2>
where
    T1: Dir<Q>,
{
    fn start_ang(&self) -> Angle<Q> {
        self.0.start_ang()
    }
    fn end_ang(&self) -> Angle<Q> {
        self.0.end_ang()
    }
}

impl<Q: Num, T> Dir<Q> for Group<T>
where
    T: Dir<Q>,
{
    fn start_ang(&self) -> Angle<Q> {
        self.0
            .first()
            .map(|x| x.start_ang())
            .unwrap_or(Angle::from_rad(Q::zero()))
    }
    fn end_ang(&self) -> Angle<Q> {
        self.0
            .first()
            .map(|x| x.end_ang())
            .unwrap_or(Angle::from_rad(Q::zero()))
    }
}

pub struct ArcCurve {
    arc: CircularArc,
    width: Vec<Length<Absolute, f64>>,
}

impl ArcCurve {
    pub fn new(arc: CircularArc, width: Vec<Length<Absolute, f64>>) -> Self {
        Self { arc, width }
    }
    pub fn into_group(
        self,
    ) -> Group<
        Area<
            impl Pos<Length<Absolute, f64>> + Dir<f64> + IntoIterator<Item = LenCo<Absolute, f64>>,
        >,
    > {
        Group(
            self.width
                .into_iter()
                .map(|x| LocatIter::locat_sweep(self.arc, (-x / 2., x / 2.)))
                .collect(),
        )
    }
}

impl Pos<Length<Absolute, f64>> for ArcCurve {
    fn start_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.arc.start_pos()
    }
    fn end_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.arc.end_pos()
    }
}

impl Pos<Length<Absolute, f64>> for Rect {
    fn start_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.line.start_pos()
    }
    fn end_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.line.end_pos()
    }
}

impl Dir<f64> for ArcCurve {
    fn start_ang(&self) -> Angle<f64> {
        self.arc.start_ang()
    }
    fn end_ang(&self) -> Angle<f64> {
        self.arc.end_ang()
    }
}

impl Dir<f64> for Rect {
    fn start_ang(&self) -> Angle<f64> {
        self.line.start_ang()
    }
    fn end_ang(&self) -> Angle<f64> {
        self.line.end_ang()
    }
}

pub(crate) struct LocatIter<Q: Quantity, A: Num, I> {
    pub(crate) pos: (Coordinate<Q>, Coordinate<Q>),
    pub(crate) ang: (Angle<A>, Angle<A>),
    pub(crate) iter: I,
}

impl<Q: Quantity, A: Num, I: Iterator<Item = Coordinate<Q>>> LocatIter<Q, A, I> {
    pub(crate) fn locat_sweep<T>(t: T, bias: (Q, Q)) -> Area<Self>
    where
        T: Sweep<Q> + Pos<Q> + Dir<A>,
        T::Output: IntoIterator<IntoIter = I>,
    {
        let pos = (t.start_pos(), t.end_pos());
        let ang = (t.start_ang(), t.end_ang());
        Area {
            area: Self {
                pos,
                ang,
                iter: t.sweep(bias).into_iter(),
            },
        }
    }
}

impl<Q: Quantity, A: Num, I: Iterator<Item = Coordinate<Q>>> Iterator for LocatIter<Q, A, I> {
    type Item = Coordinate<Q>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<Q: Quantity, A: Num, I: Iterator<Item = Coordinate<Q>> + DoubleEndedIterator>
    DoubleEndedIterator for LocatIter<Q, A, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
    fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.iter.rfind(predicate)
    }
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, f)
    }
}

impl<Q: Quantity, A: Num, I: Iterator<Item = Coordinate<Q>>> Pos<Q> for LocatIter<Q, A, I> {
    fn start_pos(&self) -> Coordinate<Q> {
        self.pos.0.clone()
    }
    fn end_pos(&self) -> Coordinate<Q> {
        self.pos.1.clone()
    }
}

impl<Q: Quantity, A: Num, I: Iterator<Item = Coordinate<Q>>> Dir<A> for LocatIter<Q, A, I> {
    fn start_ang(&self) -> Angle<A> {
        self.ang.0.clone()
    }
    fn end_ang(&self) -> Angle<A> {
        self.ang.1.clone()
    }
}
pub struct Rect {
    line: Line,
    width: Vec<Length<Absolute, f64>>,
}

impl Rect {
    pub fn new(line: Line, width: Vec<Length<Absolute, f64>>) -> Self {
        Self { line, width }
    }
    pub fn into_group(
        self,
    ) -> Group<
        Area<
            impl Pos<Length<Absolute, f64>> + Dir<f64> + IntoIterator<Item = LenCo<Absolute, f64>>,
        >,
    > {
        Group(
            self.width
                .into_iter()
                .map(|x| LocatIter::locat_sweep(self.line, (-x / 2., x / 2.)))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{zero, MICROMETER};

    use super::*;
    #[test]
    fn apply_cursor() {
        let mut c: Cursor = Cursor::new((zero(), zero()), Angle::from_deg(90f64));
        let rect = Rect::new(
            Line::new((MICROMETER, MICROMETER), (MICROMETER * 2., MICROMETER * 2.)),
            [MICROMETER / 4.].into(),
        );
        let rect = c.assemble(rect.into_group());
        
    }
}
