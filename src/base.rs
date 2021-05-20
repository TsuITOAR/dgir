#![allow(unused_variables)]
#![allow(dead_code)]
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

impl TryInto<gds21::GdsBoundary> for Polygon<f64> {
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
                .map(|x| num::ToPrimitive::to_i32(&x).unwrap())
                .collect(),
            ..Default::default()
        })
    }
}

pub trait Curve: Sized {
    fn points_iter(self) -> Box<dyn Iterator<Item = [f64; 2]>>;
    fn connect_line<T: Curve>(self, other: T) -> Box<dyn Iterator<Item = [f64; 2]>> {
        Box::new(self.points_iter().chain(other.points_iter()))
    }
}

pub trait ClosedCurve: Curve {}

pub trait HasWidth {
    fn width(&self) -> f64;
}

impl<T> From<(T, LayerData)> for Polygon<f64>
where
    T: ClosedCurve,
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
impl Circle<f64> {
    pub fn new(center: (f64, f64), radius: f64, resolution: Resolution<f64>) -> Self {
        let two_pi = 2. * std::f64::consts::PI;
        let points_num = match resolution {
            Resolution::MinDistance(d) => {
                num::ToPrimitive::to_usize(&(radius / d * two_pi)).unwrap()
            }
            Resolution::MinNumber(n) => n,
        };

        let ang_step: f64 = two_pi / num::ToPrimitive::to_f64(&(points_num - 1)).unwrap();
        let ang_iter = successors(Some(0.), |x| Some(x + ang_step))
            .take(points_num - 1)
            .chain(std::iter::once(0.));

        let point_list = ang_iter
            .map(|arg| [center.0 + radius * arg.cos(), center.1 + radius * arg.sin()])
            .collect();
        Self { xy: point_list }
    }
}

impl Curve for Circle<f64> {
    fn points_iter(self) -> Box<dyn Iterator<Item = [f64; 2]>> {
        Box::new(self.xy.into_iter())
    }
}
impl ClosedCurve for Circle<f64> {}
