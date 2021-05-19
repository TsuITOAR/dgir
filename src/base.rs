use std::{convert::TryInto, usize};

use gds21::GdsBoundary;
use ndarray;

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

struct Arrow<P, A> {
    position: (P, P),
    direction: A,
}

struct LayerData {
    layer: i16,
    datatype: i16,
}

struct Port<P, A, const N: usize> {
    vector_info: Arrow<P, A>,
    port: [(P, LayerData); N],
}

struct Polygon<T = f64> {
    xy: ndarray::Array2<T>,
    layer_data: LayerData,
}

impl<T> TryInto<gds21::GdsBoundary> for Polygon<T>
where
    T: Into<i32> + Copy,
{
    type Error = Box<dyn std::error::Error>;
    fn try_into(self) -> Result<gds21::GdsBoundary, Self::Error> {
        let shape = self.xy.shape();
        if shape[0] != 2 {
            return Err("Incompatible xy dimension".into());
        }
        let len = shape[1] * 2;
        Ok(gds21::GdsBoundary {
            layer: self.layer_data.layer,
            datatype: self.layer_data.datatype,
            xy: self
                .xy
                .into_shape((1, len * 2))?
                .into_iter()
                .map(|x: &T| -> i32 { Into::<i32>::into(*x) })
                .collect(),
            ..Default::default()
        })
    }
}

trait Curve {
    fn line<T: Into<i32>>(&self) -> ndarray::Array2<T>;
}

trait ClosedCurve: Curve {}

trait HasWidth {
    fn width<T: Into<i32>>(&self) -> T;
}

impl<T, U> From<(T, LayerData)> for Polygon<U>
where
    T: ClosedCurve + HasWidth,
    U: Into<i32>,
{
    fn from(s: (T, LayerData)) -> Self {
        let (line, layer_data) = s;
        let xy = line.line::<U>();
        Self { layer_data, xy }
    }
}

struct Circle<T>{
    xy:ndarray::Array2<T>
}
impl<T> Circle<T>{
    fn new<U:Into<T>>(center:(T,T),radius:T)
}
