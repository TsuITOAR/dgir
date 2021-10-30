use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Sub, SubAssign},
};

use crate::units::{Absolute, Angle, Length, Meter};
use gds21::GdsPoint;
use nalgebra::{Point2, Scalar};
use num::{traits::NumAssignOps, Float, FromPrimitive, Num, ToPrimitive, Zero};

pub mod curve;
pub mod transfer;
use self::elements::{Compound, IntoCurve};

pub mod elements;
pub mod coordinate;
//TO-DO:This actually cost more time for little file, need to figure out
pub(crate) type Coordinate<T> = Point2<T>;

#[derive(Debug, Clone, Copy)]
pub enum Resolution<T> {
    MinDistance(T),
    MinNumber(usize),
}

pub trait Distance:
    'static
    + Sized
    + Clone
    + Debug
    + PartialEq
    + Add<Self, Output = Self>
    + AddAssign
    + Sub<Self, Output = Self>
    + SubAssign
    + Mul<Self::Basic, Output = Self>
    + MulAssign<Self::Basic>
    + Div<Self, Output = Self::Basic>
    + DivAssign<Self::Basic>
    + Zero
{
    type Basic: Num + NumAssignOps + Float + Sized + Copy + ToPrimitive + FromPrimitive + Debug;
    fn from(meter: f64) -> Self;
    fn to_basic(self) -> Self::Basic;
    fn from_basic(b: Self::Basic) -> Self;
}

impl<S: 'static + Num + NumAssignOps + Copy + Float + ToPrimitive + FromPrimitive + Debug> Distance
    for Length<Absolute, S>
{
    type Basic = S;
    fn from(meter: f64) -> Self {
        Length::new_absolute::<Meter>(S::from_f64(meter).unwrap())
    }
    fn to_basic(self) -> Self::Basic {
        self.value
    }
    fn from_basic(b: Self::Basic) -> Self {
        Self {
            value: b,
            marker: PhantomData,
        }
    }
}
pub struct Curve<In: 'static, Out: 'static + Scalar> {
    para_list: Box<dyn Iterator<Item = In>>,
    para_equ: Box<dyn FnMut(In) -> Coordinate<Out>>,
}

impl<In: 'static + Copy, Out: 'static + Scalar> Curve<In, Out> {
    pub fn new(
        list: impl Iterator<Item = In> + 'static,
        para_equ: impl FnMut(In) -> Coordinate<Out> + 'static,
    ) -> Self {
        Self {
            para_list: Box::new(list),
            para_equ: Box::new(para_equ),
        }
    }
    pub fn draw(self) -> Drawing<Out> {
        let para_equ = self.para_equ;
        Drawing::Iter(Box::new(self.para_list.map(para_equ)))
    }
    pub fn decorate_input(mut self, decorator: impl FnMut(In) -> In + 'static) -> Self {
        self.para_list = Box::new(self.para_list.map(decorator));
        self
    }
    pub fn decorate_output(
        mut self,
        mut decorator: impl FnMut(Coordinate<Out>) -> Coordinate<Out> + 'static,
    ) -> Self {
        let mut para_equ = self.para_equ;
        self.para_equ = Box::new(move |coordinate| decorator(para_equ(coordinate)));
        self
    }
    pub fn rotate(self, angle: Angle<Out::Basic>) -> Self
    where
        Out: Distance + Copy,
    {
        let decorator = move |input: Coordinate<Out>| {
            Coordinate::from([
                input[0] * angle.to_rad().cos() - input[1] * angle.to_rad().sin(),
                input[0] * angle.to_rad().sin() + input[1] * angle.to_rad().cos(),
            ])
        };
        self.decorate_output(decorator)
    }
    pub fn move_evenly(self, x: Out, y: Out) -> Self
    where
        Out: Distance + Copy,
    {
        let decorator =
            move |input: Coordinate<Out>| Coordinate::from([input[0] + x, input[1] + y]);
        self.decorate_output(decorator)
    }
}
pub enum Drawing<T: Scalar> {
    Iter(Box<dyn Iterator<Item = Coordinate<T>>>),
    Points(Vec<Coordinate<T>>),
}
impl<T: Distance + Clone + Scalar> Drawing<T> {
    pub(crate) fn to_xy(self, database_length: T) -> Vec<GdsPoint> {
        let convert = |x: Coordinate<T>| -> GdsPoint {
            GdsPoint::new(
                (x[0].clone() / database_length.clone()).to_i32().unwrap(),
                (x[1].clone() / database_length.clone()).to_i32().unwrap(),
            )
        };
        let ret: Vec<GdsPoint> = match self {
            Drawing::Iter(iter) => iter.map(convert).collect(),
            Drawing::Points(points) => points.into_iter().map(convert).collect(),
        };
        ret
    }

    pub fn connect(self, other: Self) -> Self
    where
        T: 'static,
    {
        Drawing::Iter(match (self, other) {
            (Drawing::Iter(s), Drawing::Iter(o)) => Box::new(s.chain(o)),
            (Drawing::Iter(s), Drawing::Points(o)) => Box::new(s.chain(o.into_iter())),
            (Drawing::Points(s), Drawing::Iter(o)) => Box::new(s.into_iter().chain(o)),
            (Drawing::Points(s), Drawing::Points(o)) => {
                Box::new(s.into_iter().chain(o.into_iter()))
            }
        })
    }
    pub fn reverse(self) -> Self
    where
        T: 'static,
    {
        Drawing::Iter(Box::new(match self {
            Drawing::Iter(it) => it.collect::<Vec<Coordinate<T>>>().into_iter().rev(),
            Drawing::Points(p) => p.into_iter().rev(),
        }))
    }
}

pub trait Offset: Sized {
    type Field: Add<Self::Field, Output = Self::Field> + Copy;
    fn field(&mut self) -> &mut Self::Field;
    fn offset(mut self, change: Self::Field) -> Self {
        *self.field() = *self.field() + change;
        self
    }
    fn into_compound(self, offsets: (Self::Field, Self::Field)) -> Compound<Self, Self>
    where
        Self: IntoCurve + Clone,
    {
        let s1 = self.clone();
        Compound::new(self.offset(offsets.0), s1.offset(offsets.1))
    }
}
pub struct Broadened<C: IntoCurve + Broaden> {
    curve: C,
    width: C::Field,
}

impl<C: IntoCurve + Broaden> Broadened<C> {
    fn new(curve: C, width: C::Field) -> Self {
        Self { curve, width }
    }
}

impl<C> IntoCurve for Broadened<C>
where
    C: IntoCurve + Broaden,
    C::In: Copy,
    C::Out: Scalar,
    C::Field: Distance,
    <C::Field as Distance>::Basic: FromPrimitive,
{
    type In = (C::In, bool);
    type Out = C::Out;
    fn forward(self) -> Curve<Self::In, Self::Out> {
        self.curve
            .into_compound((
                self.width * <C::Field as Distance>::Basic::from_f64(-0.5).unwrap(),
                self.width * <C::Field as Distance>::Basic::from_f64(0.5).unwrap(),
            ))
            .forward()
    }
    fn backward(self) -> Curve<Self::In, Self::Out> {
        self.curve
            .into_compound((
                self.width * <C::Field as Distance>::Basic::from_f64(-0.5).unwrap(),
                self.width * <C::Field as Distance>::Basic::from_f64(0.5).unwrap(),
            ))
            .backward()
    }
}
pub trait Broaden: Sized + Offset + IntoCurve + Clone {
    fn set_width(self, width: Self::Field) -> Broadened<Self>
    where
        Self::Field: Distance,
    {
        Broadened::new(self, width)
    }
}

pub struct Seed<S> {
    core: S,
}
