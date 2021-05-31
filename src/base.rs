use num::{traits::FloatConst, Float, FromPrimitive, Num, ToPrimitive, Zero};

use crate::{
    paint::*,
    units::{Length, Unit},
};
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
}

pub struct Arrow<P, A> {
    position: (P, P),
    direction: A,
}

pub struct Port<P, A, const N: usize> {
    vector_info: Arrow<P, A>,
    port: [(P, LayerData); N],
} */

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

/* pub trait HasWidth {
    fn width(&self) -> f64;
} */

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