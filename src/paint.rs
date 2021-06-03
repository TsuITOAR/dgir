use crate::draw::{Drawing, Ruler};

#[derive(Clone, Copy)]
pub struct LayerData {
    pub layer: i16,
    pub datatype: i16,
}
impl LayerData {
    pub fn new(layer: i16, datatype: i16) -> Self {
        Self { layer, datatype }
    }
    pub fn color<T>(self, drawing: Drawing<T>) -> ColorDrawing<T> {
        ColorDrawing::new(self, drawing)
    }
}

pub struct ColorDrawing<T> {
    pub(crate) color: LayerData,
    pub(crate) drawing: Drawing<T>,
}

impl<T> ColorDrawing<T> {
    pub fn new(color: LayerData, drawing: Drawing<T>) -> Self {
        Self { color, drawing }
    }
}

impl<In: 'static, Out: 'static> Ruler<In, Out> {}
