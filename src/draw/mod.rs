use std::ops::{Add, Div, Mul, Sub};

use num::Num;

use crate::units::{Length, Unit};

mod elements;

pub trait Convert<T> {
    fn convert(self) -> T;
}

impl<U: Unit<S>, V: Unit<S>, S: Num + Copy> Convert<Length<V, S>> for Length<U, S> {
    fn convert(self) -> Length<V, S> {
        self.conversion()
    }
}
pub trait Brush:
    Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self::Basic, Output = Self>
    + Div<Self, Output = Self::Basic>
{
    type Basic: Num + Sized + Copy;
}

impl<U: Unit<S>, S: Num + Copy> Brush for Length<U, S> {
    type Basic = S;
}

impl Brush for f64 {
    type Basic = f64;
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
        Drawing::Iter(Box::new(self.list.map(move |p| [x(p), y(p)])))
    }
    pub fn decorate(self, decorator: Box<dyn FnMut(In) -> In>) -> Self {
        Self::new(Box::new(self.list.map(decorator)), self.x, self.y)
    }
}
pub enum Drawing<T> {
    Iter(Box<dyn Iterator<Item = [T; 2]>>),
    Points(Vec<[T; 2]>),
}
impl<U, T: Convert<U> + 'static> Convert<[U; 2]> for [T; 2] {
    fn convert(self) -> [U; 2] {
        [self[0].convert(), self[1].convert()]
    }
}
impl<U, T: Convert<U> + 'static> Convert<Drawing<U>> for Drawing<T> {
    fn convert(self) -> Drawing<U> {
        match self {
            Drawing::Iter(iter) => Drawing::Iter(Box::new(iter.map(|u| u.convert()))),
            Drawing::Points(points) => {
                Drawing::Iter(Box::new(points.into_iter().map(|u| u.convert())))
            }
        }
    }
}
