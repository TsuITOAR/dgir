use num::{traits::FloatConst, Float, FromPrimitive, Num, ToPrimitive, Zero};

use crate::units::{Length, Unit};
use std::{convert::TryInto, iter::successors};
const MAX_POINTS_NUM: usize = 8191;
trait Attachable: Sized {
    fn attach<E: Extendable>(self, e: &mut E) -> &mut E {
        e.extend(self)
    }
}

trait Extendable: Sized {
    fn extend<A: Attachable>(&mut self, a: A) -> &mut Self {
        unimplemented!();
    }
    fn adapt<A: Adapter>(&mut self, adapter: A) -> &mut Self {
        todo!()
    }
    fn connect<T: Extendable, U>(&mut self, target: T) -> U {
        unimplemented!();
    }
}

trait Adapter: Sized {
    fn reverse(self) -> Self {
        unimplemented!();
    }
}

impl<T: Adapter> Attachable for T {}

/* trait Integrated:Sized{
    fn place()
} */

pub struct Arrow<P, A> {
    position: (P, P),
    direction: A,
}

pub struct LayerData {
    layer: i16,
    datatype: i16,
}
impl LayerData {
    pub fn new(layer: i16, datatype: i16) -> Self {
        Self { layer, datatype }
    }
}

pub struct Port<P, A, const N: usize> {
    vector_info: Arrow<P, A>,
    port: [(P, LayerData); N],
}

pub struct Polygon<T> {
    xy: Vec<[T; 2]>,
    layer_data: LayerData,
}

impl<U, S> TryInto<gds21::GdsBoundary> for Polygon<Length<U, S>>
where
    U: Unit<S>,
    S: Num + ToPrimitive,
{
    type Error = Box<dyn std::error::Error>;
    fn try_into(self) -> Result<gds21::GdsBoundary, Self::Error> {
        if self.xy.len() > MAX_POINTS_NUM {
            return Err("too many points in a polygon".into());
        }
        Ok(gds21::GdsBoundary {
            layer: self.layer_data.layer,
            datatype: self.layer_data.datatype,
            xy: self
                .xy
                .into_iter()
                .flatten()
                .map(|x| num::ToPrimitive::to_i32(&x.value).unwrap())
                .collect(),
            ..Default::default()
        })
    }
}

pub trait Curve<'a, T: 'a>: Sized {
    fn points_iter(self) -> Box<dyn Iterator<Item = [T; 2]> + 'a>;
    fn connect_line<U: Curve<'a, T>>(self, other: U) -> Box<dyn Iterator<Item = [T; 2]> + 'a> {
        Box::new(self.points_iter().chain(other.points_iter()))
    }
}

pub trait ClosedCurve<'a, T: 'a>: Curve<'a, T> {}

pub trait HasWidth {
    fn width(&self) -> f64;
}

impl<'a, T, U> From<(T, LayerData)> for Polygon<U>
where
    T: ClosedCurve<'a, U>,
    U: 'a,
{
    fn from(s: (T, LayerData)) -> Self {
        let (line, layer_data) = s;
        let xy = line.points_iter().collect();
        Self { layer_data, xy }
    }
}

pub enum Resolution<T> {
    MinDistance(T),
    MinNumber(usize),
}

//TO-DO:make xy an enum composed of coordinates and iterator that return a coordinate
pub struct Circle<T> {
    xy: Vec<[T; 2]>,
}
impl<U, S> Circle<Length<U, S>>
where
    U: Unit<S>,
    S: Float + FloatConst + ToPrimitive + FromPrimitive,
{
    pub fn new(
        center: (Length<U, S>, Length<U, S>),
        radius: Length<U, S>,
        resolution: Resolution<Length<U, S>>,
    ) -> Self {
        let two_pi = <S as FloatConst>::PI() + <S as FloatConst>::PI();
        let points_num: usize = match resolution {
            Resolution::MinDistance(d) => (radius / d * two_pi).to_usize().unwrap(),
            Resolution::MinNumber(n) => n,
        };

        let ang_step = two_pi / <S as FromPrimitive>::from_usize(points_num).unwrap();
        let ang_iter = successors(Some(<S as Zero>::zero()), |x| Some(*x + ang_step))
            .take(points_num)
            .chain(std::iter::once(<S as Zero>::zero()));

        let point_list = ang_iter
            .map(|ang| [center.0 + radius * ang.cos(), center.1 + radius * ang.sin()])
            .collect();
        Self { xy: point_list }
    }
}

impl<'a, U, S> Curve<'a, Length<U, S>> for Circle<Length<U, S>>
where
    U: Unit<S>,
    S: Float + FloatConst + ToPrimitive + FromPrimitive,
    Length<U, S>: 'a,
{
    fn points_iter(self) -> Box<dyn Iterator<Item = [Length<U, S>; 2]> + 'a> {
        Box::new(self.xy.into_iter())
    }
}
impl<'a, U, S> ClosedCurve<'a, Length<U, S>> for Circle<Length<U, S>>
where
    U: Unit<S>,
    S: Float + FloatConst + ToPrimitive + FromPrimitive,
    Length<U, S>: 'a,
{
}
