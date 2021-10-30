use std::{iter::successors, ops::Add};

use num::{traits::FloatConst, Float, FromPrimitive, ToPrimitive, Zero};

use crate::{draw::Resolution, units::Angle};

use super::{Broaden, Coordinate, Curve, Distance, Offset};

pub trait IntoCurve {
    type In: 'static;
    type Out: 'static + Distance;
    fn forward(self) -> Curve<Self::In, Self::Out>;
    fn backward(self) -> Curve<Self::In, Self::Out>;
}

#[derive(Debug, Clone)]
pub struct Compound<R1, R2> {
    forward: R1,
    backward: R2,
}

impl<R1, R2> Compound<R1, R2> {
    pub fn new(forward: R1, backward: R2) -> Self {
        Self { forward, backward }
    }
    pub fn compound_with<R3>(self, other: R3) -> Compound<Self, R3> {
        Compound::<Self, R3>::new(self, other)
    }
}

impl<R1, R2> IntoCurve for Compound<R1, R2>
where
    R1: IntoCurve,
    R1::In: Copy,
    R2: IntoCurve<In = R1::In, Out = R1::Out>,
{
    type In = (R1::In, bool);
    type Out = R1::Out;
    fn forward(self) -> Curve<Self::In, Self::Out> {
        let Curve {
            para_list: list1,
            para_equ: mut equ1,
        } = self.forward.forward();
        let Curve {
            para_list: list2,
            para_equ: mut equ2,
        } = self.backward.backward();
        let list = CompoundIter::new(list1, list2);
        let para_equ = move |input: Self::In| {
            let para = input.0;
            match input.1 {
                true => equ1(para),
                false => equ2(para),
            }
        };
        Curve::new(list, para_equ)
    }
    fn backward(self) -> Curve<Self::In, Self::Out> {
        let Curve {
            para_list: list1,
            para_equ: mut equ1,
        } = self.forward.backward();
        let Curve {
            para_list: list2,
            para_equ: mut equ2,
        } = self.backward.forward();
        let list = CompoundIter::new(list2, list1);
        let para_equ = move |input: Self::In| {
            let para = input.0;
            match input.1 {
                true => equ2(para),
                false => equ1(para),
            }
        };
        Curve::new(list, para_equ)
    }
}

struct CompoundIter<I1, I2> {
    stage1: I1,
    stage2: I2,
    flag: bool,
}

impl<I1, I2> CompoundIter<I1, I2> {
    fn new(stage1: I1, stage2: I2) -> Self {
        Self {
            stage1,
            stage2,
            flag: true,
        }
    }
}

impl<I1, I2> Iterator for CompoundIter<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
    type Item = (I1::Item, bool);
    fn next(&mut self) -> Option<Self::Item> {
        match self.flag {
            true => {
                if let Some(v) = self.stage1.next() {
                    Some((v, self.flag))
                } else {
                    self.flag = false;
                    if let Some(v) = self.stage2.next() {
                        Some((v, self.flag))
                    } else {
                        None
                    }
                }
            }
            false => {
                if let Some(v) = self.stage2.next() {
                    Some((v, self.flag))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

impl<S> IntoCurve for Circle<S>
where
    S: 'static + Distance + Copy,
    <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type In = <S as Distance>::Basic;
    type Out = S;
    fn forward(self) -> Curve<Self::In, Self::Out> {
        let two_pi = <Self::In as FloatConst>::PI() + <Self::In as FloatConst>::PI();
        let step_num = match self.resolution {
            Resolution::MinNumber(n) => n - 1,
            Resolution::MinDistance(d) => (two_pi * (self.radius / d).abs()).to_usize().unwrap(),
        };
        let ang_step =
            two_pi / <<S as Distance>::Basic as FromPrimitive>::from_usize(step_num).unwrap();
        let list = successors(Some(<Self::In as Zero>::zero()), move |ang| {
            Some(*ang + ang_step)
        })
        .take(step_num + 1);
        let radius = self.radius;
        let center = self.center;
        let x = move |ang: Self::In| center.0 + radius * ang.cos();
        let y = move |ang: Self::In| center.1 + radius * ang.sin();
        let para_equ = move |ang: Self::In| Coordinate::from([x(ang), y(ang)]);
        Curve::new(list, para_equ)
    }
    fn backward(self) -> Curve<Self::In, Self::Out> {
        self.forward()
    }
}

impl<S> Offset for Circle<S>
where
    S: Copy + Add<Output = S>,
{
    type Field = S;
    fn field(&mut self) -> &mut Self::Field {
        &mut self.radius
    }
}

impl<S: Sized + Copy + Add<Output = S> + Distance + 'static> Broaden for Circle<S> where
    S::Basic: FloatConst
{
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle<S: Copy> {
    x: S,
    y: S,
}

impl<S: Copy> Rectangle<S> {
    pub fn new(x: S, y: S) -> Self
    where
        S: Distance,
    {
        Self { x, y }
    }
}

impl<S> IntoCurve for Rectangle<S>
where
    S: 'static + Distance + Copy,
    S::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type In = (S, S);
    type Out = S;
    fn forward(self) -> Curve<Self::In, Self::Out> {
        let points = vec![
            (
                self.x * S::Basic::from_f64(-0.5).unwrap(),
                self.y * S::Basic::from_f64(-0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(0.5).unwrap(),
                self.y * S::Basic::from_f64(-0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(0.5).unwrap(),
                self.y * S::Basic::from_f64(0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(-0.5).unwrap(),
                self.y * S::Basic::from_f64(0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(-0.5).unwrap(),
                self.y * S::Basic::from_f64(-0.5).unwrap(),
            ),
        ];
        let para_equ = move |point: Self::In| Coordinate::from([point.0, point.1]);
        Curve::new(points.into_iter(), para_equ)
    }
    fn backward(self) -> Curve<Self::In, Self::Out> {
        let points = vec![
            (
                self.x * S::Basic::from_f64(-0.5).unwrap(),
                self.y * S::Basic::from_f64(-0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(0.5).unwrap(),
                self.y * S::Basic::from_f64(-0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(0.5).unwrap(),
                self.y * S::Basic::from_f64(0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(-0.5).unwrap(),
                self.y * S::Basic::from_f64(0.5).unwrap(),
            ),
            (
                self.x * S::Basic::from_f64(-0.5).unwrap(),
                self.y * S::Basic::from_f64(-0.5).unwrap(),
            ),
        ];
        let para_equ = move |point: Self::In| Coordinate::from([point.0, point.1]);
        Curve::new(points.into_iter().rev(), para_equ)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CircularArc<S, A> {
    center: (S, S),
    radius: S,
    angle: (Angle<A>, Angle<A>),
    resolution: Resolution<S>,
}

impl<S> CircularArc<S, S::Basic>
where
    S: Distance,
{
    pub fn new(
        center: (S, S),
        radius: S,
        angle: (Angle<S::Basic>, Angle<S::Basic>),
        resolution: Resolution<S>,
    ) -> Self {
        Self {
            center,
            radius,
            angle,
            resolution,
        }
    }
}

impl<S> IntoCurve for CircularArc<S, S::Basic>
where
    S: 'static + Distance + Copy,
    <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type In = <S as Distance>::Basic;
    type Out = S;
    fn forward(self) -> Curve<Self::In, Self::Out> {
        let (rad1, rad2) = (self.angle.0.to_rad(), self.angle.1.to_rad());
        let diff_angle = rad2 - rad1;
        let step_num = match self.resolution {
            Resolution::MinNumber(n) => n - 1,
            Resolution::MinDistance(d) => {
                ((diff_angle * (self.radius / d)).abs()).to_usize().unwrap()
            }
        };
        let ang_step =
            diff_angle / <<S as Distance>::Basic as FromPrimitive>::from_usize(step_num).unwrap();
        let list = successors(Some(rad1), move |ang| Some(*ang + ang_step)).take(step_num + 1);
        let radius = self.radius;
        let center = self.center;
        let x = move |ang: Self::In| center.0 + radius * ang.cos();
        let y = move |ang: Self::In| center.1 + radius * ang.sin();
        let para_equ = move |ang: Self::In| Coordinate::from([x(ang), y(ang)]);
        Curve::new(list, para_equ)
    }
    fn backward(self) -> Curve<Self::In, Self::Out> {
        let (rad2, rad1) = (self.angle.0.to_rad(), self.angle.1.to_rad());
        let diff_angle = rad2 - rad1;
        let step_num = match self.resolution {
            Resolution::MinNumber(n) => n - 1,
            Resolution::MinDistance(d) => {
                ((diff_angle * (self.radius / d)).abs()).to_usize().unwrap()
            }
        };
        let ang_step =
            diff_angle / <<S as Distance>::Basic as FromPrimitive>::from_usize(step_num).unwrap();
        let list = successors(Some(rad1), move |ang| Some(*ang + ang_step)).take(step_num + 1);
        let radius = self.radius;
        let center = self.center;
        let x = move |ang: Self::In| center.0 + radius * ang.cos();
        let y = move |ang: Self::In| center.1 + radius * ang.sin();
        let para_equ = move |ang: Self::In| Coordinate::from([x(ang), y(ang)]);
        Curve::new(list, para_equ)
    }
}

impl<S> Offset for CircularArc<S, S::Basic>
where
    S: 'static + Distance + Copy,
    <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type Field = S;
    fn field(&mut self) -> &mut Self::Field {
        &mut self.radius
    }
}

impl<S: Sized + Copy + Add<Output = S> + Distance + 'static> Broaden for CircularArc<S, S::Basic> where
    S::Basic: FloatConst
{
}
