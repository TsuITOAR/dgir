use std::iter::{once, successors};

use num::{traits::FloatConst, Float, FromPrimitive, ToPrimitive, Zero};

use crate::draw::Resolution;

use super::{Brush, Ruler};

pub trait RulerFactory {
    type In: 'static;
    type Out: 'static + Brush;
    fn produce(self) -> Ruler<Self::In, Self::Out>;
}

pub struct Circle<S> {
    pub center: (S, S),
    pub radius: S,
    pub resolution: Resolution<S>,
}

impl<S> Circle<S> {
    pub fn new(center: (S, S), radius: S, resolution: Resolution<S>) -> Self {
        Self {
            center,
            radius,
            resolution,
        }
    }
}

impl<S> RulerFactory for Circle<S>
where
    S: 'static + Brush + Copy,
    <S as Brush>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type In = <S as Brush>::Basic;
    type Out = S;
    fn produce(self) -> Ruler<Self::In, Self::Out> {
        let two_pi = <Self::In as FloatConst>::PI() + <Self::In as FloatConst>::PI();
        let step_num = match self.resolution {
            Resolution::MinNumber(n) => n,
            Resolution::MinDistance(d) => (two_pi * (self.radius / d).abs()).to_usize().unwrap(),
        };
        let ang_step =
            two_pi / <<S as Brush>::Basic as FromPrimitive>::from_usize(step_num).unwrap();
        let list = successors(Some(<Self::In as Zero>::zero()), move |ang| {
            Some(*ang + ang_step)
        })
        .take(step_num)
        .chain(once(<Self::In as Zero>::zero()));
        let radius = self.radius;
        let center = self.center;
        let x = move |ang: Self::In| center.0 + radius * ang.cos();
        let y = move |ang: Self::In| center.1 + radius * ang.sin();
        Ruler::new(list, x, y)
    }
}
