use std::ops::{Add, Div, Mul, Sub};

use crate::units::{AbsoluteLength, Length, LengthType, MakeLength, Meter};
use arrayvec::ArrayVec;
use num::{FromPrimitive, Num, ToPrimitive};

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
pub trait Brush:
    Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self::Basic, Output = Self>
    + Div<Self, Output = Self::Basic>
{
    type Basic: Num + Sized + Copy + ToPrimitive;
    fn from(meter: f64) -> Self;
}

impl<S: Num + Copy + ToPrimitive+FromPrimitive> Brush for Length<AbsoluteLength<S>, S> {
    type Basic = S;
    fn from(meter: f64) -> Self {
        MakeLength::<Meter, S>::new_absolute(S::from_f64(meter).unwrap())
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
impl<T: Brush + Clone> Drawing<T> {
    pub(crate) fn to_xy(self, database_length: T) -> Vec<i32> {
        let convert = |x: T| (x / database_length.clone()).to_i32().unwrap();
        match self {
            Drawing::Iter(iter) => iter.flatten().map(convert).collect(),
            Drawing::Points(points) => points.into_iter().flatten().map(convert).collect(),
        }
    }
}
/* impl<U, T: Clone + Convert<U> + 'static> Convert<Coordinate<U>> for Coordinate<T> {
    fn convert(self) -> Coordinate<U> {
        self.into_iter().map(|x| x.convert()).collect()
    }
}
impl<U, T: Clone + Convert<U> + 'static> Convert<Drawing<U>> for Drawing<T> {
    fn convert(self) -> Drawing<U> {
        match self {
            Drawing::Iter(iter) => Drawing::Iter(Box::new(iter.map(|u| u.convert()))),
            Drawing::Points(points) => {
                Drawing::Iter(Box::new(points.into_iter().map(|u| u.convert())))
            }
        }
    }
}
 */
