use crate::gds::{Element, Path, Polygon};

#[derive(Clone, Copy, Debug, Default)]
pub struct LayerData {
    pub(crate) layer: i16,
    pub(crate) datatype: i16,
}
impl LayerData {
    pub fn new(layer: i16, datatype: i16) -> Self {
        Self { layer, datatype }
    }
}
/* pub trait Color: Copy {
    fn to_brush<T: Distance + Copy>(self, width: T) -> Brush<T, Self> {
        Brush {
            width: Some(width),
            color: self,
        }
    }
    fn to_line<T: Distance + Copy>(self) -> Brush<T, Self> {
        Brush {
            width: None,
            color: self,
        }
    }
    fn to_filler(self) -> Filler<Self> {
        Filler { color: self }
    }
} */

/* #[derive(Debug, Clone, Copy)]
pub struct Brush<T: Distance + Copy, C: Color> {
    width: Option<T>,
    color: C,
}

impl<T: Distance + Copy, C: Color> Brush<T, C> {
    pub fn new(color: C) -> Self {
        Self { color, width: None }
    }
    pub fn set_width(&mut self, width: T) -> &mut Self {
        self.width = Some(width);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Filler<C: Color + Copy> {
    color: C,
}
impl Color for LayerData {}

pub trait Shader<T: Distance>: Copy {
    type Output;
    fn color(self, drawing: Drawing<T>) -> Self::Output;
}

impl<T: Distance + Copy> Shader<T> for Brush<T, LayerData> {
    type Output = Painting<T>;
    fn color(self, drawing: Drawing<T>) -> Self::Output {
        Painting::Path(Path {
            coordinates: drawing,
            color: self.color,
            width: self.width,
        })
    }
}

impl<T: Distance + Copy> Shader<T> for Filler<LayerData> {
    type Output = Painting<T>;
    fn color(self, drawing: Drawing<T>) -> Self::Output {
        Painting::Polygon(Polygon {
            coordinates: drawing,
            color: self.color,
        })
    }
} */
