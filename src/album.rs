use std::{
    collections::BTreeSet,
    iter::Empty,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use gds21::GdsPoint;
use nalgebra::Scalar;
use num::{FromPrimitive, Num, ToPrimitive};

use crate::{
    close_curve,
    color::LayerData,
    draw::coordinate::{Coordinate, LenCo},
    points_num_check,
    units::{Absolute, Length, LengthType, Relative},
};

pub struct Path<L: LengthType, T: Num + Scalar> {
    pub curve: Box<dyn Iterator<Item = LenCo<L, T>>>,
    pub color: LayerData,
    pub width: Option<Length<L, T>>,
}
impl<L, T> Path<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    pub fn to_dgircell(self) -> Element<L, T>
    where
        T: Clone,
    {
        Element::Path(self)
    }
}

pub struct Polygon<L: LengthType, T: Num + Scalar> {
    pub area: Box<dyn Iterator<Item = LenCo<L, T>>>,
    pub color: LayerData,
}
impl<L, T> Polygon<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    pub fn to_dgircell(self) -> Element<L, T>
    where
        T: Clone,
    {
        Element::Polygon(self)
    }
}

pub struct Ref<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    pub(crate) decorator: Option<gds21::GdsStrans>,
    pub(crate) position: Coordinate<Length<L, T>>,
    pub(crate) reference: String,
    pub(crate) dependencies: Option<BTreeSet<Rc<DgirCell<L, T>>>>,
}

pub enum Element<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    Path(Path<L, T>),
    Polygon(Polygon<L, T>),
    Ref(Ref<L, T>),
}

impl<L, T> From<Path<L, T>> for Element<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn from(p: Path<L, T>) -> Self {
        Element::Path(p)
    }
}

impl<L, T> From<Polygon<L, T>> for Element<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn from(p: Polygon<L, T>) -> Self {
        Element::Polygon(p)
    }
}

impl<L, T> From<Ref<L, T>> for Element<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn from(r: Ref<L, T>) -> Self {
        Element::Ref(r)
    }
}
pub struct DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    pub name: String,
    pub(crate) elements: Vec<Element<L, T>>,
}

impl<L, T> DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            elements: Vec::new(),
        }
    }
    pub fn rename(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn insert<U: Into<Element<L, T>>>(&mut self, element: U) -> &mut Self {
        self.elements.push(element.into());
        self
    }

    /* pub fn as_ref(self) -> Ref<L, T> {
        self.into()
    } */

    pub fn get_dependencies(&mut self) -> BTreeSet<Rc<DgirCell<L, T>>> {
        let mut dependencies = BTreeSet::new();
        for element in self.elements.iter_mut() {
            match element {
                Element::Ref(Ref {
                    dependencies: Some(ref mut d),
                    ..
                }) => {
                    dependencies.append(d);
                }
                _ => (),
            }
        }
        dependencies
    }
}
trait ToGdsPoints: Iterator {
    type Scale;
    fn to_gdspoints(self, scale: Self::Scale) -> Vec<GdsPoint>;
}

impl<I, T> ToGdsPoints for I
where
    T: Num + Scalar + ToPrimitive,
    I: Iterator<Item = LenCo<Absolute, T>>,
{
    type Scale = Length<Absolute, T>;
    fn to_gdspoints(self, scale: Self::Scale) -> Vec<GdsPoint> {
        self.map(|x| {
            GdsPoint::new(
                (x[0].clone() / scale.clone()).to_i32().unwrap(),
                (x[1].clone() / scale.clone()).to_i32().unwrap(),
            )
        })
        .collect()
    }
}

/* impl<I, T> ToGdsPoints for I
where
    I: Iterator<Item = LenCo<Relative, T>>,
{
    type Scale = ();
    fn to_gdspoints(self, _: ()) -> Vec<GdsPoint> {
        self.map(|x| GdsPoint::new(x[0].to_i32().unwrap(), x[1].to_i32().unwrap()))
    }
} */

impl<T> DgirCell<Absolute, T>
where
    T: Num + Scalar + ToPrimitive,
{
    pub fn to_gds(self, database_unit: Length<Absolute, T>) -> gds21::GdsStruct {
        use gds21::*;
        let mut new_cell = GdsStruct::new(self.name);
        for painting in self.elements {
            new_cell.elems.push(match painting {
                Element::Path(p) => GdsElement::GdsPath({
                    let xy = p.curve.to_gdspoints(database_unit.clone());
                    points_num_check(&xy);
                    GdsPath {
                        layer: p.color.layer,
                        datatype: p.color.datatype,
                        xy,
                        width: match p.width {
                            Some(l) => (l / database_unit.clone()).to_i32(),
                            None => None,
                        },
                        ..Default::default()
                    }
                }),
                Element::Polygon(p) => GdsElement::GdsBoundary({
                    let mut xy = p.area.to_gdspoints(database_unit.clone());
                    debug_assert!(close_curve(&mut xy));
                    debug_assert!(points_num_check(&xy));
                    GdsBoundary {
                        layer: p.color.layer,
                        datatype: p.color.datatype,
                        xy,
                        ..Default::default()
                    }
                }),
                Element::Ref(r) => GdsElement::GdsStructRef(GdsStructRef {
                    name: r.reference,
                    xy: GdsPoint::new(
                        (r.position[0].clone() / database_unit.clone())
                            .to_i32()
                            .unwrap(),
                        (r.position[1].clone() / database_unit.clone())
                            .to_i32()
                            .unwrap(),
                    ),
                    strans: r.decorator,
                    ..Default::default()
                }),
            })
        }
        new_cell
    }
}

impl<L, T> PartialEq for DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
impl<L, T> Eq for DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
}

impl<L, T> PartialOrd for DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
impl<L, T> Ord for DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<L, T> AsRef<Vec<Element<L, T>>> for DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn as_ref(&self) -> &Vec<Element<L, T>> {
        &self.elements
    }
}

impl<L, T> AsMut<Vec<Element<L, T>>> for DgirCell<L, T>
where
    L: LengthType,
    T: Num + Scalar,
{
    fn as_mut(&mut self) -> &mut Vec<Element<L, T>> {
        &mut self.elements
    }
}
