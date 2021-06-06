use std::{iter::successors, ops::Add};

use num::{traits::FloatConst, Float, FromPrimitive, ToPrimitive, Zero};

use crate::{draw::Resolution, units::Angle};

use super::{Coordinate, Distance, Ruler};

pub trait RulerFactory {
    type In: 'static;
    type Out: 'static + Distance;
    fn produce(self) -> Ruler<Self::In, Self::Out>;
    fn reverse(self) -> Ruler<Self::In, Self::Out>;
}

#[derive(Debug, Clone)]
pub struct Compound<R1, R2> {
    r1: R1,
    r2: R2,
}

impl<R1, R2> Compound<R1, R2> {
    pub fn new(r1: R1, r2: R2) -> Self {
        Self { r1, r2 }
    }
}

impl<R1, R2> RulerFactory for Compound<R1, R2>
where
    R1: RulerFactory,
    R1::In: Copy,
    R2: RulerFactory<In = R1::In, Out = R1::Out>,
{
    type In = (R1::In, bool);
    type Out = R1::Out;
    fn produce(self) -> Ruler<Self::In, Self::Out> {
        let Ruler {
            para_list: list1,
            para_equ: mut equ1,
        } = self.r1.produce();
        let Ruler {
            para_list: list2,
            para_equ: mut equ2,
        } = self.r2.reverse();
        let list = CompoundIter::new(list1, list2);
        let para_equ = move |input: Self::In| {
            let para = input.0;
            match input.1 {
                true => equ1(para),
                false => equ2(para),
            }
        };
        Ruler::new(list, para_equ)
    }
    fn reverse(self) -> Ruler<Self::In, Self::Out> {
        let Ruler {
            para_list: list1,
            para_equ: mut equ1,
        } = self.r1.reverse();
        let Ruler {
            para_list: list2,
            para_equ: mut equ2,
        } = self.r2.produce();
        let list = CompoundIter::new(list2, list1);
        let para_equ = move |input: Self::In| {
            let para = input.0;
            match input.1 {
                true => equ2(para),
                false => equ1(para),
            }
        };
        Ruler::new(list, para_equ)
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

pub trait Offset: Sized {
    type Field: Add<Self::Field, Output = Self::Field> + Copy;
    fn field(&mut self) -> &mut Self::Field;
    fn offset(mut self, change: Self::Field) -> Self {
        *self.field() = *self.field() + change;
        self
    }
    fn into_compound(self, offsets: (Self::Field, Self::Field)) -> Compound<Self, Self>
    where
        Self: RulerFactory + Clone,
    {
        let s1 = self.clone();
        Compound::new(self.offset(offsets.0), s1.offset(offsets.1))
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
        Ruler::new(list, para_equ)
    }
    fn reverse(self) -> Ruler<Self::In, Self::Out> {
        self.produce()
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

impl<S> RulerFactory for Rectangle<S>
where
    S: 'static + Distance + Copy,
    S::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type In = (S, S);
    type Out = S;
    fn produce(self) -> Ruler<Self::In, Self::Out> {
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
        Ruler::new(points.into_iter(), para_equ)
    }
    fn reverse(self) -> Ruler<Self::In, Self::Out> {
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
        Ruler::new(points.into_iter().rev(), para_equ)
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

impl<S> RulerFactory for CircularArc<S, S::Basic>
where
    S: 'static + Distance + Copy,
    <S as Distance>::Basic: FloatConst + Float + ToPrimitive + FromPrimitive,
{
    type In = <S as Distance>::Basic;
    type Out = S;
    fn produce(self) -> Ruler<Self::In, Self::Out> {
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
        Ruler::new(list, para_equ)
    }
    fn reverse(self) -> Ruler<Self::In, Self::Out> {
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
        Ruler::new(list, para_equ)
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
