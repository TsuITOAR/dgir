use crate::draw::{Convert, Drawing};

#[derive(Clone, Copy)]
pub struct LayerData {
    pub layer: i16,
    pub datatype: i16,
}
impl LayerData {
    pub fn new(layer: i16, datatype: i16) -> Self {
        Self { layer, datatype }
    }
    pub fn color<T>(&self, drawing: Drawing<T>) -> ColorDrawing<T> {
        ColorDrawing::new(self.clone(), drawing)
    }
}

pub struct ColorDrawing<T> {
    color: LayerData,
    drawing: Drawing<T>,
}

impl<T> ColorDrawing<T> {
    pub fn new(color: LayerData, drawing: Drawing<T>) -> Self {
        Self { color, drawing }
    }
}

impl<U, T: Convert<U> + 'static> Convert<ColorDrawing<U>> for ColorDrawing<T> {
    fn convert(self) -> ColorDrawing<U> {
        ColorDrawing::<U> {
            color: self.color,
            drawing: self.drawing.convert(),
        }
    }
}
