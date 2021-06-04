use std::{
    iter::{once, successors},
    ops::Add,
};

use num::{traits::FloatConst, Float, FromPrimitive, ToPrimitive, Zero};

use crate::{
    draw::Resolution,
    units::{Angle, Deg},
};

use super::{Distance, Ruler};

pub trait RulerFactory {
    type In: 'static;
    type Out: 'static + Distance;
    fn produce(self) -> Ruler<Self::In, Self::Out>;
}

pub struct Circle<S> {
    center: (S, S),
    radius: S,
    resolution: Resolution<S>,
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
    S: 'static + Distance + Copy,
    <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type In = <S as Distance>::Basic;
    type Out = S;
    fn produce(self) -> Ruler<Self::In, Self::Out> {
        let two_pi = <Self::In as FloatConst>::PI() + <Self::In as FloatConst>::PI();
        let step_num = match self.resolution {
            Resolution::MinNumber(n) => n,
            Resolution::MinDistance(d) => (two_pi * (self.radius / d).abs()).to_usize().unwrap(),
        };
        let ang_step =
            two_pi / <<S as Distance>::Basic as FromPrimitive>::from_usize(step_num).unwrap();
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

pub struct Rectangle<S: Copy, A: Angle + Copy = Deg<S>> {
    point1: (S, S),
    point2: (S, S),
    angle: Option<A>,
}

impl<S: Copy, A: Angle + Copy> Rectangle<S, A> {
    pub fn from_points(point1: (S, S), point2: (S, S)) -> Self {
        Self {
            point1,
            point2,
            angle: None,
        }
    }
    pub fn from_lens(x: S, y: S) -> Self
    where
        S: Distance,
    {
        Self {
            point1: (
                x * <S as Distance>::Basic::from_f64(-0.5).unwrap(),
                y * <S as Distance>::Basic::from_f64(-0.5).unwrap(),
            ),
            point2: (
                x * <S as Distance>::Basic::from_f64(0.5).unwrap(),
                y * <S as Distance>::Basic::from_f64(0.5).unwrap(),
            ),
            angle: None,
        }
    }
    pub fn rotate<U: Into<A>>(&mut self, angle: U)
    where
        A: Add<Output = A>,
    {
        match self.angle {
            Some(ref mut a) => *a = a.clone() + angle.into(),
            None => self.angle = Some(angle.into()),
        }
    }
}

impl<S, A> RulerFactory for Rectangle<S, A>
where
    S: 'static + Distance + Copy,
    <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
    A: 'static + Angle<Basic = <S as Distance>::Basic> + Copy,
{
    type In = (S, S);
    type Out = S;
    fn produce(self) -> Ruler<Self::In, Self::Out> {
        let points = vec![
            (self.point1.0, self.point1.1),
            (self.point2.0, self.point1.1),
            (self.point2.0, self.point2.1),
            (self.point1.0, self.point2.1),
            (self.point1.0, self.point1.1),
        ];
        let (x, y): (
            Box<dyn FnMut(Self::In) -> Self::Out>,
            Box<dyn FnMut(Self::In) -> Self::Out>,
        ) = match self.angle {
            Some(a) => (
                Box::new(move |point: (S, S)| {
                    point.0 * a.to_rad().cos() - point.1 * a.to_rad().sin()
                }),
                Box::new(move |point: (S, S)| {
                    point.0 * a.to_rad().sin() + point.1 * a.to_rad().cos()
                }),
            ),
            None => (
                Box::new(move |point: (S, S)| point.0),
                Box::new(move |point: (S, S)| point.1),
            ),
        };
        Ruler {
            list: Box::new(points.into_iter()),
            x,
            y,
        }
    }
}

pub struct CircularArc<S, A> {
    center: (S, S),
    radius: S,
    angle: (A, A),
    resolution: Resolution<S>,
}

impl<S, A> CircularArc<S, A> {
    pub fn new(center: (S, S), radius: S, angle: (A, A), resolution: Resolution<S>) -> Self {
        Self {
            center,
            radius,
            angle,
            resolution,
        }
    }
}

impl<S, A> RulerFactory for CircularArc<S, A>
where
    S: 'static + Distance + Copy,
    <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
    A: 'static + Angle<Basic = <S as Distance>::Basic> + Copy,
{
    type In = <S as Distance>::Basic;
    type Out = S;
    fn produce(self) -> Ruler<Self::In, Self::Out> {
        let (rad1, rad2) = (self.angle.0.to_rad(), self.angle.1.to_rad());
        let diff_angle = rad2 - rad1;
        let step_num = match self.resolution {
            Resolution::MinNumber(n) => n,
            Resolution::MinDistance(d) => {
                ((diff_angle * (self.radius / d)).abs()).to_usize().unwrap()
            }
        };
        let ang_step =
            diff_angle / <<S as Distance>::Basic as FromPrimitive>::from_usize(step_num).unwrap();
        let list = successors(Some(rad1), move |ang| Some(*ang + ang_step)).take(step_num);
        let radius = self.radius;
        let center = self.center;
        let x = move |ang: Self::In| center.0 + radius * ang.cos();
        let y = move |ang: Self::In| center.1 + radius * ang.sin();
        Ruler::new(list, x, y)
    }
}
