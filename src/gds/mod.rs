use std::{collections::BTreeSet, ops::Index, rc::Rc};

use gds21::GdsPoint;
use nalgebra::Scalar;
use num::{Num, ToPrimitive, Zero};

use crate::{
    close_curve,
    color::LayerData,
    draw::coordinate::{Coordinate, LenCo},
    points_num_check,
    units::{Absolute, Length, LengthType, Relative},
};

type Points<L, T> = Box<dyn Iterator<Item = LenCo<L, T>>>;

pub struct Path<L: LengthType, T: Num + Scalar> {
    pub curve: Points<L, T>,
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
    pub area: Points<L, T>,
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
    pub(crate) strans: Option<gds21::GdsStrans>,
    pub(crate) pos: Coordinate<Length<L, T>>,
    pub(crate) id: String,
    pub(crate) dep: BTreeSet<Rc<DgirCell<L, T>>>, //TODO need to avoid circular ref, or dead loop happens
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

    pub fn as_ref(self) -> Ref<L, T> {
        let mut s = self;
        Ref {
            strans: None,
            dep: s.get_dependencies(),
            pos: Coordinate::from([Length::zero(), Length::zero()]),
            id: s.name,
        }
    }
    //make sure every sub dependencies is empty
    pub(crate) fn get_dependencies(&mut self) -> BTreeSet<Rc<DgirCell<L, T>>> {
        let mut dependencies = BTreeSet::new();
        for element in self.elements.iter_mut() {
            match element {
                Element::Ref(Ref { dep: ref mut d, .. }) => {
                    debug_assert!(is_sub_dependencies_empty(d));
                    dependencies.append(d);
                }
                _ => (),
            }
        }
        dependencies
    }
}

fn is_sub_dependencies_empty<L: LengthType, T: Scalar + Num>(
    set: &BTreeSet<Rc<DgirCell<L, T>>>,
) -> bool {
    set.iter().all(|c| {
        c.elements.iter().all(|e| match e {
            Element::Ref(Ref {
                dep: dependencies, ..
            }) => dependencies.is_empty(),
            _ => true,
        })
    })
}

trait ToGdsPoints: Iterator {
    type Scale: Clone;
    fn to_gdspoints(self, scale: Self::Scale) -> Vec<GdsPoint>;
}

//helper trait to constrain type of iterators which give different type of length
//see https://stackoverflow.com/questions/34470995/how-to-allow-multiple-implementations-of-a-trait-on-various-types-of-intoiterato
//see https://github.com/rust-lang/rust/issues/31844
//see https://stackoverflow.com/questions/40392524/conflicting-trait-implementations-even-though-associated-types-differ
trait CoordinateIterator {
    type Length;
    type Scale: Clone;
    type Scalar: ToPrimitive;
    type Coordinate: Index<usize, Output = Self::Length>;
    fn after_scale(coor: Self::Coordinate, scale: Self::Scale) -> GdsPoint;
}

impl<C, T: Scalar + Num + ToPrimitive> CoordinateIterator for (C, LenCo<Absolute, T>) {
    type Length = Length<Absolute, T>;
    type Scale = Length<Absolute, T>;
    type Scalar = T;
    type Coordinate = LenCo<Absolute, T>;
    fn after_scale(coor: Self::Coordinate, scale: Self::Scale) -> GdsPoint {
        GdsPoint {
            x: (coor[0].clone() / scale.clone()).to_i32().unwrap(),
            y: (coor[1].clone() / scale.clone()).to_i32().unwrap(),
        }
    }
}

impl<C, T: Scalar + Num + ToPrimitive> CoordinateIterator for (C, LenCo<Relative, T>) {
    type Length = Length<Relative, T>;
    type Scale = ();
    type Scalar = T;
    type Coordinate = LenCo<Relative, T>;
    fn after_scale(coor: Self::Coordinate, _: Self::Scale) -> GdsPoint {
        GdsPoint {
            x: (coor[0].value).to_i32().unwrap(),
            y: (coor[1].value).to_i32().unwrap(),
        }
    }
}

impl<I> ToGdsPoints for I
where
    I: Iterator,
    (I, I::Item): CoordinateIterator<Coordinate = I::Item>,
{
    type Scale = <(I, I::Item) as CoordinateIterator>::Scale;
    fn to_gdspoints(self, scale: Self::Scale) -> Vec<GdsPoint> {
        self.map(|x| <(I, I::Item)>::after_scale(x, scale.clone()))
            .collect()
    }
}

trait ToGdsStruct {
    fn to_gdsstruct(self) -> gds21::GdsStruct;
}

impl<T> DgirCell<Absolute, T>
where
    T: Num + Scalar + ToPrimitive,
{
    pub fn to_gds(self, database_len: Length<Absolute, T>) -> gds21::GdsStruct {
        use gds21::*;
        let mut new_cell = GdsStruct::new(self.name);
        for painting in self.elements {
            new_cell.elems.push(match painting {
                Element::Path(p) => GdsElement::GdsPath({
                    let xy = p.curve.to_gdspoints(database_len.clone());
                    points_num_check(&xy);
                    GdsPath {
                        layer: p.color.layer,
                        datatype: p.color.datatype,
                        xy,
                        width: match p.width {
                            Some(l) => (l / database_len.clone()).to_i32(),
                            None => None,
                        },
                        ..Default::default()
                    }
                }),
                Element::Polygon(p) => GdsElement::GdsBoundary({
                    let mut xy = p.area.to_gdspoints(database_len.clone());
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
                    name: r.id,
                    xy: GdsPoint::new(
                        (r.pos[0].clone() / database_len.clone()).to_i32().unwrap(),
                        (r.pos[1].clone() / database_len.clone()).to_i32().unwrap(),
                    ),
                    strans: r.strans,
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
