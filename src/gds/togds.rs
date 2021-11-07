use gds21::GdsPoint as Gds21Point;
use nalgebra::Scalar;
use num::{FromPrimitive, Num, ToPrimitive};
use std::{ops::Index, rc::Rc};

use crate::{
    close_curve,
    draw::coordinate::LenCo,
    gds::Element,
    points_num_check,
    units::{Absolute, Length, Relative},
};

use super::DgirCell;

pub(crate) trait ToGds21Points: Iterator {
    type Scale: Clone;
    fn to_gds21_points(self, scale: Self::Scale) -> Vec<Gds21Point>;
}

//helper trait to constrain type of iterators which give different type of length
//see https://stackoverflow.com/questions/34470995/how-to-allow-multiple-implementations-of-a-trait-on-various-types-of-intoiterato
//see https://github.com/rust-lang/rust/issues/31844
//see https://stackoverflow.com/questions/40392524/conflicting-trait-implementations-even-though-associated-types-differ
pub(crate) trait CoordinateIterator {
    type Length;
    type Scale: Clone;
    type Scalar: ToPrimitive;
    type Coordinate: Index<usize, Output = Self::Length>;
    fn after_scale(coor: Self::Coordinate, scale: Self::Scale) -> Gds21Point;
}

impl<C, T: Scalar + Num + ToPrimitive> CoordinateIterator for (C, LenCo<Absolute, T>) {
    type Length = Length<Absolute, T>;
    type Scale = Length<Absolute, T>;
    type Scalar = T;
    type Coordinate = LenCo<Absolute, T>;
    fn after_scale(coor: Self::Coordinate, scale: Self::Scale) -> Gds21Point {
        Gds21Point {
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
    fn after_scale(coor: Self::Coordinate, _: Self::Scale) -> Gds21Point {
        Gds21Point {
            x: (coor[0].value).to_i32().unwrap(),
            y: (coor[1].value).to_i32().unwrap(),
        }
    }
}

impl<I> ToGds21Points for I
where
    I: Iterator,
    (I, I::Item): CoordinateIterator<Coordinate = I::Item>,
{
    type Scale = <(I, I::Item) as CoordinateIterator>::Scale;
    fn to_gds21_points(self, scale: Self::Scale) -> Vec<Gds21Point> {
        self.map(|x| <(I, I::Item)>::after_scale(x, scale.clone()))
            .collect()
    }
}

pub(crate) trait ToGds21Struct {
    type Scale;
    fn to_gds21_struct(self, scale: Self::Scale) -> gds21::GdsStruct;
}

impl<T> ToGds21Struct for DgirCell<Absolute, T>
where
    T: Num + Scalar + ToPrimitive,
{
    type Scale = Length<Absolute, T>;
    fn to_gds21_struct(self, scale: Self::Scale) -> gds21::GdsStruct {
        use gds21::*;
        let mut new_cell = GdsStruct::new(self.name);
        for painting in self.elements {
            new_cell.elems.push(match painting {
                Element::Path(p) => GdsElement::GdsPath({
                    let xy = p.curve.to_gds21_points(scale.clone());
                    points_num_check(&xy);
                    GdsPath {
                        layer: p.color.layer,
                        datatype: p.color.datatype,
                        xy,
                        width: match p.width {
                            Some(l) => (l / scale.clone()).to_i32(),
                            None => None,
                        },
                        ..Default::default()
                    }
                }),
                Element::Polygon(p) => GdsElement::GdsBoundary({
                    let mut xy = p.area.to_gds21_points(scale.clone());
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
                    xy: Gds21Point::new(
                        (r.pos[0].clone() / scale.clone()).to_i32().unwrap(),
                        (r.pos[1].clone() / scale.clone()).to_i32().unwrap(),
                    ),
                    strans: r.strans,
                    ..Default::default()
                }),
            })
        }
        new_cell
    }
}

impl<T> ToGds21Struct for DgirCell<Relative, T>
where
    T: Num + Scalar + ToPrimitive,
{
    type Scale = ();
    fn to_gds21_struct(self, scale: Self::Scale) -> gds21::GdsStruct {
        use gds21::*;
        let mut new_cell = GdsStruct::new(self.name);
        for painting in self.elements {
            new_cell.elems.push(match painting {
                Element::Path(p) => GdsElement::GdsPath({
                    let xy = p.curve.to_gds21_points(scale.clone());
                    points_num_check(&xy);
                    GdsPath {
                        layer: p.color.layer,
                        datatype: p.color.datatype,
                        xy,
                        width: match p.width {
                            Some(l) => l.value.to_i32(),
                            None => None,
                        },
                        ..Default::default()
                    }
                }),
                Element::Polygon(p) => GdsElement::GdsBoundary({
                    let mut xy = p.area.to_gds21_points(scale.clone());
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
                    xy: Gds21Point::new(
                        r.pos[0].value.to_i32().unwrap(),
                        r.pos[1].value.to_i32().unwrap(),
                    ),
                    strans: r.strans,
                    ..Default::default()
                }),
            })
        }
        new_cell
    }
}

pub(crate) trait ToGds21Library {
    fn to_gds21_library(self) -> gds21::GdsLibrary;
}

impl<T> ToGds21Library for super::DgirLibrary<Absolute, T>
where
    T: Num + Scalar + ToPrimitive + FromPrimitive,
{
    fn to_gds21_library(self) -> gds21::GdsLibrary {
        gds21::GdsLibrary {
            name: self.name.unwrap_or(
                self.cells
                    .first()
                    .map(|x| x.name.clone())
                    .unwrap_or_default(),
            ),
            structs: {
                let database_unit = self.units.database.clone();
                let mut cells = self.cells.into_iter();
                match cells.next() {
                    None => Vec::new(),
                    Some(mut first_cell) => {
                        let mut dependencies = first_cell.get_dependencies();
                        for mut cell in cells {
                            //if only one topcell is expected, all of its dependencies should be inside itself
                            dependencies.append(&mut cell.get_dependencies());
                            dependencies.insert(Rc::new(cell));
                        }
                        let mut structs = Vec::with_capacity(dependencies.len() + 1);
                        debug_assert!(!dependencies.contains(&first_cell));
                        structs.push(first_cell.to_gds21_struct(database_unit.clone()));
                        for s in dependencies {
                            structs.push(
                                Rc::try_unwrap(s)
                                    .expect("all rc points should be in set, which makes it unique")
                                    .to_gds21_struct(database_unit.clone()),
                            );
                        }
                        structs
                    }
                }
            },
            units: gds21::GdsUnits::new(
                (self.units.database.clone() / self.units.user)
                    .to_f64()
                    .unwrap(),
                (self.units.database / Length::new_absolute::<crate::units::Meter>(T::one()))
                    .to_f64()
                    .unwrap(),
            ),
            ..Default::default()
        }
    }
}

impl<T> ToGds21Library for super::DgirLibrary<Relative, T>
where
    T: Num + Scalar + ToPrimitive + FromPrimitive,
{
    fn to_gds21_library(self) -> gds21::GdsLibrary {
        gds21::GdsLibrary {
            name: self.name.unwrap_or(
                self.cells
                    .first()
                    .map(|x| x.name.clone())
                    .unwrap_or_default(),
            ),
            structs: {
                let mut cells = self.cells.into_iter();
                match cells.next() {
                    None => Vec::new(),
                    Some(mut first_cell) => {
                        let mut dependencies = first_cell.get_dependencies();
                        for mut cell in cells {
                            //if only one topcell is expected, all of its dependencies should be inside itself
                            dependencies.append(&mut cell.get_dependencies());
                            dependencies.insert(Rc::new(cell));
                        }
                        let mut structs = Vec::with_capacity(dependencies.len() + 1);
                        debug_assert!(!dependencies.contains(&first_cell));
                        structs.push(first_cell.to_gds21_struct(()));
                        for s in dependencies {
                            structs.push(
                                Rc::try_unwrap(s)
                                    .expect("all rc points should be in set, which makes it unique")
                                    .to_gds21_struct(()),
                            );
                        }
                        structs
                    }
                }
            },
            units: gds21::GdsUnits::new(
                (self.units.database.clone() / self.units.user)
                    .to_f64()
                    .unwrap(),
                (self.units.database / Length::new_absolute::<crate::units::Meter>(T::one()))
                    .to_f64()
                    .unwrap(),
            ),
            ..Default::default()
        }
    }
}
