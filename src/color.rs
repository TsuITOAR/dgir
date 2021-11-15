use std::fmt::Display;

use crate::{
    draw::curve::groups::{Compound, Group},
    gds::ElementsGroup,
    Quantity,
};

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

pub trait Colour: Sized + Clone {
    fn color<D: Decorated<Self>>(self, d: D) -> ElementsGroup<D::Quantity> {
        d.color(self)
    }
}

impl Colour for LayerData {}
impl<T1: Colour, T2: Colour> Colour for Compound<T1, T2> {}
impl<T: Colour> Colour for Group<T> {}
impl<T: Colour, const LEN: usize> Colour for [T; LEN] {}

pub trait Decorated<C: Colour> {
    type Quantity: Quantity;
    fn color(self, c: C) -> ElementsGroup<Self::Quantity>;
}

impl<Q, T1, T2, C1, C2> Decorated<Compound<C1, C2>> for Compound<T1, T2>
where
    Q: Quantity,
    C1: Colour,
    C2: Colour,
    T1: Decorated<C1, Quantity = Q>,
    T2: Decorated<C2, Quantity = Q>,
{
    type Quantity = Q;
    fn color(self, c: Compound<C1, C2>) -> ElementsGroup<Self::Quantity> {
        let mut s = self.0.color(c.0);
        s.extend(self.1.color(c.1));
        s
    }
}

impl<Q, T, C> Decorated<Group<C>> for Group<T>
where
    Q: Quantity,
    C: Colour,
    T: Decorated<C, Quantity = Q>,
{
    type Quantity = Q;
    fn color(self, c: Group<C>) -> ElementsGroup<Self::Quantity> {
        self.0
            .into_iter()
            .zip(c.0.into_iter())
            .map(|x| x.0.color(x.1))
            .fold(ElementsGroup::default(), |mut accum, new| {
                accum.extend(new);
                accum
            })
    }
}

impl<Q, T, C, const LEN: usize> Decorated<[C; LEN]> for [T; LEN]
where
    Q: Quantity,
    C: Colour,
    T: Decorated<C, Quantity = Q>,
{
    type Quantity = Q;
    fn color(self, c: [C; LEN]) -> ElementsGroup<Self::Quantity> {
        self.into_iter()
            .zip(c.into_iter())
            .map(|x| x.0.color(x.1))
            .fold(ElementsGroup::default(), |mut accum, new| {
                accum.extend(new);
                accum
            })
    }
}
