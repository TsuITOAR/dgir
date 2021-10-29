use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::units::{Absolute, Angle, Length, Meter};
use arrayvec::ArrayVec;
use num::{traits::NumAssignOps, Float, FromPrimitive, Num, ToPrimitive, Zero};

pub mod curve;
pub mod elements;

//TO-DO:This actually cost more time for little file, need to figure out
pub(crate) type Coordinate<T> = ArrayVec<T, 2>;

#[derive(Debug, Clone, Copy)]
pub enum Resolution<T> {
    MinDistance(T),
    MinNumber(usize),
}

pub trait Distance:
    Sized
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
    type Basic: Num + NumAssignOps + Float + Sized + Copy + ToPrimitive + FromPrimitive;
    fn from(meter: f64) -> Self;
}

impl<S: Num + NumAssignOps + Copy + Float + ToPrimitive + FromPrimitive> Distance
    for Length<Absolute, S>
{
    type Basic = S;
    fn from(meter: f64) -> Self {
        Length::new_absolute::<Meter>(S::from_f64(meter).unwrap())
    }
}
pub struct Ruler<In: 'static, Out: 'static> {
    para_list: Box<dyn Iterator<Item = In>>,
    para_equ: Box<dyn FnMut(In) -> Coordinate<Out>>,
}

impl<In: 'static + Copy, Out: 'static> Ruler<In, Out> {
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
pub enum Drawing<T> {
    Iter(Box<dyn Iterator<Item = Coordinate<T>>>),
    Points(Vec<Coordinate<T>>),
}
impl<T: Distance + Clone> Drawing<T> {
    pub(crate) fn to_xy(self, database_length: T) -> Vec<i32> {
        let convert = |x: T| (x / database_length.clone()).to_i32().unwrap();
        let ret: Vec<i32> = match self {
            Drawing::Iter(iter) => iter.flatten().map(convert).collect(),
            Drawing::Points(points) => points.into_iter().flatten().map(convert).collect(),
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
