use std::ops::{Add, Div, Mul, Sub};

use crate::units::{Absolute, Length, Meter};
use arrayvec::ArrayVec;
use num::{FromPrimitive, Num, ToPrimitive, Zero};

pub mod elements;

//TO-DO:This actually cost more time for little file, need to figure out
pub(crate) type Coordinate<T> = ArrayVec<T, 2>;
pub enum Resolution<T> {
    MinDistance(T),
    MinNumber(usize),
}
/* pub trait Convert<T> {
    fn convert(self) -> T;
}

impl<U: LengthType<S>, V: LengthType<S>, S: Num + Copy> Convert<MakeLength<V, S>> for MakeLength<U, S> {
    fn convert(self) -> MakeLength<V, S> {
        self.conversion()
    }
} */
pub trait Distance:
    Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self::Basic, Output = Self>
    + Div<Self, Output = Self::Basic>
    + Zero
{
    type Basic: Num + Sized + Copy + ToPrimitive + FromPrimitive;
    fn from(meter: f64) -> Self;
}

impl<S: Num + Copy + ToPrimitive + FromPrimitive> Distance for Length<Absolute, S> {
    type Basic = S;
    fn from(meter: f64) -> Self {
        Length::new_absolute::<Meter>(S::from_f64(meter).unwrap())
    }
}
pub struct Ruler<In: 'static, Out: 'static> {
    list: Box<dyn Iterator<Item = In>>,
    x: Box<dyn FnMut(In) -> Out>,
    y: Box<dyn FnMut(In) -> Out>,
}

impl<In: 'static + Copy, Out: 'static> Ruler<In, Out> {
    pub fn new(
        list: impl Iterator<Item = In> + 'static,
        x: impl FnMut(In) -> Out + 'static,
        y: impl FnMut(In) -> Out + 'static,
    ) -> Self {
        Self {
            list: Box::new(list),
            x: Box::new(x),
            y: Box::new(y),
        }
    }
    pub fn draw(self) -> Drawing<Out> {
        let mut x = self.x;
        let mut y = self.y;
        Drawing::Iter(Box::new(
            self.list.map(move |p| Coordinate::from([x(p), y(p)])),
        ))
    }
    pub fn decorate(self, decorator: Box<dyn FnMut(In) -> In>) -> Self {
        Self::new(Box::new(self.list.map(decorator)), self.x, self.y)
    }
}
pub enum Drawing<T> {
    Iter(Box<dyn Iterator<Item = Coordinate<T>>>),
    Points(Vec<Coordinate<T>>),
}
impl<T: Distance + Clone> Drawing<T> {
    pub(crate) fn to_xy(self, database_length: T) -> Vec<i32> {
        let convert = |x: T| (x / database_length.clone()).to_i32().unwrap();
        match self {
            Drawing::Iter(iter) => iter.flatten().map(convert).collect(),
            Drawing::Points(points) => points.into_iter().flatten().map(convert).collect(),
        }
    }
}
