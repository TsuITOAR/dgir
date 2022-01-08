use std::mem;

use nalgebra::{RealField, Rotation, Translation};
use num::{traits::FloatConst, Float, FromPrimitive, Zero};

use crate::{
    color::{Colour, LayerData},
    draw::{
        coordinate::{Coordinate, LenCo, MulAsScalar},
        curve::{
            groups::{Compound, Group},
            Area, Curve, Sweep,
        },
        transfer::{CommonTrans, MulOpClosure, Transfer},
        CircularArc, Line, Resolution,
    },
    gds::DgirCell,
    units::{Absolute, Angle, Length, LengthType},
    zero, Num, Quantity,
};

#[derive(Clone, Copy, Debug)]
pub struct Cursor<L = Absolute, T = f64>
where
    L: LengthType,
    T: Num,
{
    pub pos: LenCo<L, T>,
    pub dir: Angle<T>,
}

impl<L: LengthType, T: Num> Default for Cursor<L, T> {
    fn default() -> Self {
        Self {
            pos: LenCo::from([Zero::zero(), Zero::zero()]),
            dir: Angle::from_rad(Zero::zero()),
        }
    }
}

impl<L: LengthType, T: Num> Cursor<L, T> {
    pub fn new<C: Into<LenCo<L, T>>>(pos: C, dir: Angle<T>) -> Self {
        Self {
            pos: pos.into(),
            dir,
        }
    }
    pub fn with_cell<Cell: AsMut<DgirCell<Length<L, T>>>, C: Colour>(
        self,
        cell: Cell,
        color: C,
    ) -> CellCursor<Cell, C, L, T> {
        CellCursor {
            cell,
            color,
            cursor: self,
        }
    }
    //WHAT THE FUCK
    pub fn assemble<E>(&mut self, e: E) -> <<<E as Transfer<Length<L, T>>>::Output<MulOpClosure<MulAsScalar<Translation<T, 2_usize>>>> as Transfer<Length<L, T>>>::Output<MulOpClosure<MulAsScalar<Rotation<T, 2_usize>>>> as Transfer<Length<L, T>>>::Output<MulOpClosure<MulAsScalar<Translation<T, 2_usize>>>>
    where
        T: RealField + Float,
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
        self.pos = self.pos + (end_pos - start_pos).rotate(self.dir - start_ang);
        self.dir = self.dir + end_ang - start_ang;
        e
    }
}

#[derive(Debug)]
pub struct CellCursor<
    Cell: AsMut<DgirCell<Length<L, T>>>,
    C: Colour + Clone,
    L: LengthType = Absolute,
    T: Num = f64,
> {
    pub cursor: Cursor<L, T>,
    cell: Cell,
    pub color: C,
}

impl<C: Colour, L: LengthType, T: Num> CellCursor<DgirCell<Length<L, T>>, C, L, T> {
    pub fn new<S: ToString>(cell_name: S, color: C) -> Self {
        Self {
            cursor: Cursor::default(),
            cell: DgirCell::new(cell_name),
            color,
        }
    }
}

impl<Cell: AsMut<DgirCell<Length<L, T>>>, C: Colour, L: LengthType, T: Num>
    CellCursor<Cell, C, L, T>
{
    pub fn new_from_cell(cell: Cell, color: C) -> Self {
        Self {
            cursor: Cursor::default(),
            cell,
            color,
        }
    }
    pub fn with_assembler<W: AsRef<[Length<L, T>]>>(
        self,
        width: W,
        res: Resolution,
    ) -> Assembler<Cell, W, L, T>
    where
        C: Into<Group<LayerData>>,
    {
        Assembler {
            cell_cur: CellCursor {
                cell: self.cell,
                color: self.color.into(),
                cursor: self.cursor,
            },
            res,
            width,
        }
    }
    pub fn mut_cell(&mut self) -> &mut DgirCell<Length<L, T>> {
        self.cell.as_mut()
    }
    pub fn into_cell(self) -> Cell {
        self.cell
    }
    pub fn assemble_in<E>(&mut self, e: E) -> &mut Self
    where
        T: RealField + Float,
        E: CommonTrans<L, T> + Pos<Length<L, T>> + Dir<T>,
        <<<E as Transfer<Length<L, T>>>::Output<MulOpClosure<MulAsScalar<Translation<T, 2_usize>>>> as Transfer<Length<L, T>>>::Output<MulOpClosure<MulAsScalar<Rotation<T, 2_usize>>>> as Transfer<Length<L, T>>>::Output<MulOpClosure<MulAsScalar<Translation<T, 2_usize>>>>:crate::color::Decorated<C,Quantity=Length<L,T>>,
    {
        self.cell
            .as_mut()
            .push(self.color.clone().color(self.cursor.assemble(e)));
        self
    }
}

#[derive(Debug)]
pub struct Assembler<
    Cell: AsMut<DgirCell<Length<L, T>>>,
    W: AsRef<[Length<L, T>]>,
    L: LengthType = Absolute,
    T: Num = f64,
> {
    pub cell_cur: CellCursor<Cell, Group<LayerData>, L, T>,
    pub width: W,
    pub res: Resolution,
}

impl<W: AsRef<[Length<L, T>]>, L: LengthType, T: Num> Assembler<DgirCell<Length<L, T>>, W, L, T> {
    pub fn new<S: ToString, C: Into<Group<LayerData>>>(
        cell_name: S,
        color: C,
        width: W,
        res: Resolution,
    ) -> Self {
        Self {
            cell_cur: CellCursor {
                cursor: Cursor::default(),
                cell: DgirCell::new(cell_name),
                color: color.into(),
            },
            width,
            res,
        }
    }
}
impl<Cell: AsMut<DgirCell<Length<L, T>>>, W: AsRef<[Length<L, T>]>, L: LengthType, T: Num>
    Assembler<Cell, W, L, T>
{
    pub fn new_from_cell(cell: Cell, color: Group<LayerData>, width: W, res: Resolution) -> Self {
        Self {
            cell_cur: CellCursor {
                cursor: Cursor::default(),
                cell,
                color,
            },
            width,
            res,
        }
    }
    pub fn mut_cell(&mut self) -> &mut DgirCell<Length<L, T>> {
        self.cell_cur.mut_cell()
    }
    pub fn into_cell(self) -> Cell {
        self.cell_cur.cell
    }
    pub fn set_pos<P: Into<Coordinate<Length<L, T>>>>(&mut self, p: P) -> &mut Self {
        self.cell_cur.cursor.pos = p.into();
        self
    }
    pub fn set_dir(&mut self, a: Angle<T>) -> &mut Self {
        self.cell_cur.cursor.dir = a;
        self
    }
}
impl<
        Cell: AsMut<DgirCell<Length<Absolute, f64>>>,
        W: 'static + Clone + AsRef<[Length<Absolute, f64>]>,
    > Assembler<Cell, W, Absolute, f64>
{
    pub fn turn(&mut self, radius: Length<Absolute, f64>, a: Angle<f64>) -> &mut Self {
        self.cell_cur.assemble_in(
            ArcCurve::new(
                CircularArc::new_origin(radius, (Angle::from_deg(0.), a), self.res),
                self.width.clone(),
            )
            .into_group(),
        );
        self
    }
    pub fn extend(&mut self, len: Length<Absolute, f64>) -> &mut Self {
        assert!(len.is_positive());
        self.cell_cur
            .assemble_in(Rect::from_length(len, self.width.clone()).into_group());
        self
    }
    pub fn taper(&mut self, len: Length<Absolute, f64>, width: W) -> &mut Self {
        use crate::color::Decorated;
        let g = Group::from(
            self.width
                .as_ref()
                .iter()
                .zip(width.as_ref().iter())
                .map(|(w1, w2)| {
                    Area {
                        area: [
                            Coordinate::from([zero(), *w1 / 2.]),
                            Coordinate::from([zero(), -*w1 / 2.]),
                            Coordinate::from([len, -*w2 / 2.]),
                            Coordinate::from([len, *w2 / 2.]),
                            Coordinate::from([zero(), *w1 / 2.]),
                        ],
                    }
                    .rotate(self.cell_cur.cursor.dir)
                    .translate(self.cell_cur.cursor.pos[0], self.cell_cur.cursor.pos[1])
                })
                .collect::<Vec<_>>(),
        );
        self.cell_cur
            .cell
            .as_mut()
            .push(g.color(self.cell_cur.color.clone()));
        self.cell_cur.cursor.pos = self.cell_cur.cursor.pos
            + Coordinate::from([len, zero()]).rotate(self.cell_cur.cursor.dir);
        self.width = width;
        self
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

impl<L: LengthType, T: Num + Float + FromPrimitive + FloatConst> Pos<Length<L, T>>
    for CircularArc<L, T>
{
    fn start_pos(&self) -> Coordinate<Length<L, T>> {
        (
            self.inner.radius() * self.angle.0.cos() + self.center[0],
            self.inner.radius() * self.angle.0.sin() + self.center[1],
        )
            .into()
    }
    fn end_pos(&self) -> Coordinate<Length<L, T>> {
        (
            self.inner.radius() * self.angle.1.to_rad().cos() + self.center[0],
            self.inner.radius() * self.angle.1.to_rad().sin() + self.center[1],
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

#[derive(Clone, Copy, Debug)]
pub struct ArcCurve<W: AsRef<[Length<Absolute, f64>]>> {
    arc: CircularArc,
    width: W,
}

impl<W: AsRef<[Length<Absolute, f64>]>> ArcCurve<W> {
    pub fn new(arc: CircularArc, width: W) -> Self {
        Self { arc, width }
    }
    pub fn rev(mut self) -> Self {
        mem::swap(&mut self.arc.angle.0, &mut self.arc.angle.1);
        self
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
                .as_ref()
                .into_iter()
                .map(|x| LocatIter::locat_sweep(self.arc, (-*x / 2., *x / 2.)))
                .collect(),
        )
    }
    pub fn arc_mut(&mut self) -> &mut CircularArc {
        &mut self.arc
    }
}

impl<W: AsRef<[Length<Absolute, f64>]>> Pos<Length<Absolute, f64>> for ArcCurve<W> {
    fn start_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.arc.start_pos()
    }
    fn end_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.arc.end_pos()
    }
}

impl<W: AsRef<[Length<Absolute, f64>]>> Pos<Length<Absolute, f64>> for Rect<W> {
    fn start_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.line.start_pos()
    }
    fn end_pos(&self) -> Coordinate<Length<Absolute, f64>> {
        self.line.end_pos()
    }
}

impl<W: AsRef<[Length<Absolute, f64>]>> Dir<f64> for ArcCurve<W> {
    fn start_ang(&self) -> Angle<f64> {
        self.arc.start_ang()
    }
    fn end_ang(&self) -> Angle<f64> {
        self.arc.end_ang()
    }
}

impl<W: AsRef<[Length<Absolute, f64>]>> Dir<f64> for Rect<W> {
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
    fn rfold<B, F>(self, init: B, f: F) -> B
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

#[derive(Clone, Copy, Debug)]
pub struct Rect<W: AsRef<[Length<Absolute, f64>]>> {
    line: Line,
    width: W,
}

impl<W: AsRef<[Length<Absolute, f64>]>> Rect<W> {
    pub fn new(line: Line, width: W) -> Self {
        Self { line, width }
    }
    pub fn from_length(length: Length<Absolute, f64>, width: W) -> Self {
        Self {
            line: Line {
                start: [zero(), zero()].into(),
                end: [length, zero()].into(),
            },
            width,
        }
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
                .as_ref()
                .into_iter()
                .map(|x| LocatIter::locat_sweep(self.line, (-*x / 2., *x / 2.)))
                .collect(),
        )
    }
    pub fn line_mut(&mut self) -> &mut Line {
        &mut self.line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        draw::{Resolution, APPROX_EQ_MARGIN},
        zero, MICROMETER, NANOMETER,
    };
    use float_cmp::ApproxEq;
    #[test]
    fn apply_cursor() {
        let mut c: Cursor = Cursor::new((zero(), zero()), Angle::from_deg(90f64));
        let rect = Rect::new(
            Line::new((MICROMETER, MICROMETER), (MICROMETER * 2., MICROMETER * 2.)),
            [MICROMETER / 2.],
        );
        let rect: Vec<_> = c.assemble(rect.into_group()).into_iter().collect();
        assert!(c
            .pos
            .approx_eq([zero(), MICROMETER * (2.).sqrt(),].into(), APPROX_EQ_MARGIN));
        assert!(c.dir.approx_eq(Angle::from_deg(90f64), APPROX_EQ_MARGIN));
        let expected: [Coordinate<_>; 4] = [
            [MICROMETER / 4., zero()].into(),
            [MICROMETER / 4., MICROMETER * (2.).sqrt()].into(),
            [-MICROMETER / 4., MICROMETER * (2.).sqrt()].into(),
            [-MICROMETER / 4., zero()].into(),
        ];
        assert!(rect
            .iter()
            .zip(expected.iter())
            .all(|(l, r)| { l.approx_eq(*r, APPROX_EQ_MARGIN) },));

        let radius = MICROMETER * 120.;

        let circ = ArcCurve::new(
            CircularArc::new(
                radius,
                (MICROMETER, MICROMETER),
                (Angle::from_deg(30.), Angle::from_deg(60.)),
                Resolution::MinDistance(NANOMETER * 20.),
            ),
            [MICROMETER * 2., MICROMETER * 4.],
        );
        let _circ = c.assemble(circ.into_group());
        assert!(c.pos.approx_eq(
            LenCo::from([zero(), MICROMETER * (2.).sqrt()])
                + LenCo::from([
                    -radius * (1. - Angle::from_deg(30.).cos()),
                    radius * Angle::from_deg(30.).sin()
                ]),
            APPROX_EQ_MARGIN
        ))
    }
}
