use std::fmt::Display;

#[derive(Clone, Copy, Debug, Default)]
pub struct LayerData {
    pub(crate) layer: i16,
    pub(crate) datatype: i16,
}

impl Display for LayerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.layer, self.datatype)
    }
}

impl LayerData {
    pub fn new(layer: i16, datatype: i16) -> Self {
        Self { layer, datatype }
    }
}
