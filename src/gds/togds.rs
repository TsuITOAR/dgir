use gds21::GdsPoint as Gds21Point;
use num::{FromPrimitive, ToPrimitive};
use std::{ops::Index, rc::Rc};

use crate::{
    close_curve,
    draw::coordinate::{Coordinate, LenCo},
    gds::Element,
    points_num_check, split_path, split_polygon,
    units::{Absolute, Length, Relative},
    Num, MAX_POINTS_NUM,
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
    type Scale: Copy;
    type Scalar: ToPrimitive;
    type Coordinate: Index<usize, Output = Self::Length>;
    fn after_scale(coor: Self::Coordinate, scale: Self::Scale) -> Gds21Point;
}

impl<C, T: Num> CoordinateIterator for (C, LenCo<Absolute, T>) {
    type Length = Length<Absolute, T>;
    type Scale = Length<Absolute, T>;
    type Scalar = T;
    type Coordinate = LenCo<Absolute, T>;
    fn after_scale(coor: Self::Coordinate, scale: Self::Scale) -> Gds21Point {
        Gds21Point {
            x: (coor[0] / scale).to_i32().unwrap(),
            y: (coor[1] / scale).to_i32().unwrap(),
        }
    }
}

impl<C, T: Num> CoordinateIterator for (C, LenCo<Relative, T>) {
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
        self.map(|x| <(I, I::Item)>::after_scale(x, scale))
            .collect()
    }
}

pub(crate) trait ToGds21Struct {
    type Scale;
    fn to_gds21_struct(self, scale: Self::Scale) -> gds21::GdsStruct;
}

impl<T> ToGds21Struct for DgirCell<Length<Absolute, T>>
where
    T: Num,
{
    type Scale = Length<Absolute, T>;
    fn to_gds21_struct(self, scale: Self::Scale) -> gds21::GdsStruct {
        use gds21::*;
        fn to_gds_point<T: ToPrimitive + Num>(
            f: Coordinate<Length<Absolute, T>>,
            scale: Length<Absolute, T>,
        ) -> Gds21Point {
            Gds21Point::new(
                (f[0] / scale).to_i32().unwrap(),
                (f[1] / scale).to_i32().unwrap(),
            )
        }

        let mut new_cell = GdsStruct::new(self.name);
        for elem in self.elements {
            match elem {
                Element::Path(p) => {
                    let xy = p.curve.to_gds21_points(scale);
                    if points_num_check(&xy) {
                        new_cell.elems.push(GdsElement::GdsPath({
                            GdsPath {
                                layer: p.color.layer,
                                datatype: p.color.datatype,
                                xy,
                                width: match p.width {
                                    Some(l) => (l / scale).to_i32(),
                                    None => None,
                                },
                                ..Default::default()
                            }
                        }))
                    } else {
                        let width = match p.width {
                            Some(l) => (l / scale).to_i32(),
                            None => None,
                        };
                        for c in split_path(xy, MAX_POINTS_NUM) {
                            new_cell.elems.push(GdsElement::GdsPath({
                                GdsPath {
                                    layer: p.color.layer,
                                    datatype: p.color.datatype,
                                    xy: c,
                                    width,
                                    ..Default::default()
                                }
                            }))
                        }
                    }
                }
                Element::Polygon(p) => {
                    let mut xy = p.area.to_gds21_points(scale);
                    close_curve(&mut xy); //TODO: add a notify layer data if curve not closed

                    if points_num_check(&xy) {
                        new_cell.elems.push(GdsElement::GdsBoundary({
                            GdsBoundary {
                                layer: p.color.layer,
                                datatype: p.color.datatype,
                                xy,
                                ..Default::default()
                            }
                        }))
                    } else {
                        for c in split_polygon(xy, MAX_POINTS_NUM) {
                            new_cell.elems.push(GdsElement::GdsBoundary({
                                GdsBoundary {
                                    layer: p.color.layer,
                                    datatype: p.color.datatype,
                                    xy: c,
                                    ..Default::default()
                                }
                            }))
                        }
                    }
                }
                Element::Ref(r) => new_cell.elems.push(GdsElement::GdsStructRef(GdsStructRef {
                    name: r.id,
                    xy: to_gds_point(r.pos, scale),
                    strans: r.strans,
                    ..Default::default()
                })),
                Element::ARef(ar) => new_cell.elems.push(GdsElement::GdsArrayRef(GdsArrayRef {
                    name: ar.id,
                    xy: [
                        to_gds_point(ar.start, scale),
                        to_gds_point(ar.col_end, scale),
                        to_gds_point(ar.row_end, scale),
                    ],
                    cols: ar.rows,
                    rows: ar.cols,
                    strans: ar.strans,
                    ..Default::default()
                })),
            }
        }
        new_cell
    }
}

impl<T> ToGds21Struct for DgirCell<Length<Relative, T>>
where
    T: Num,
{
    type Scale = ();
    fn to_gds21_struct(self, scale: Self::Scale) -> gds21::GdsStruct {
        use gds21::*;
        fn to_gds_point<T: ToPrimitive + Num>(
            f: Coordinate<Length<Relative, T>>,
            _scale: (),
        ) -> Gds21Point {
            Gds21Point::new(
                (f[0].value).to_i32().unwrap(),
                (f[1].value).to_i32().unwrap(),
            )
        }

        let mut new_cell = GdsStruct::new(self.name);
        for elem in self.elements {
            match elem {
                Element::Path(p) => {
                    let xy = p.curve.to_gds21_points(scale);
                    if points_num_check(&xy) {
                        new_cell.elems.push(GdsElement::GdsPath({
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
                        }))
                    } else {
                        let width = match p.width {
                            Some(l) => l.value.to_i32(),
                            None => None,
                        };
                        for c in split_path(xy, MAX_POINTS_NUM) {
                            new_cell.elems.push(GdsElement::GdsPath({
                                GdsPath {
                                    layer: p.color.layer,
                                    datatype: p.color.datatype,
                                    xy: c,
                                    width,
                                    ..Default::default()
                                }
                            }))
                        }
                    }
                }
                Element::Polygon(p) => {
                    let mut xy = p.area.to_gds21_points(scale);
                    close_curve(&mut xy); //TODO: add a notify layer data if curve not closed

                    if points_num_check(&xy) {
                        new_cell.elems.push(GdsElement::GdsBoundary({
                            GdsBoundary {
                                layer: p.color.layer,
                                datatype: p.color.datatype,
                                xy,
                                ..Default::default()
                            }
                        }))
                    } else {
                        for c in split_polygon(xy, MAX_POINTS_NUM) {
                            new_cell.elems.push(GdsElement::GdsBoundary({
                                GdsBoundary {
                                    layer: p.color.layer,
                                    datatype: p.color.datatype,
                                    xy: c,
                                    ..Default::default()
                                }
                            }))
                        }
                    }
                }
                Element::Ref(r) => new_cell.elems.push(GdsElement::GdsStructRef(GdsStructRef {
                    name: r.id,
                    xy: to_gds_point(r.pos, scale),
                    strans: r.strans,
                    ..Default::default()
                })),
                Element::ARef(ar) => new_cell.elems.push(GdsElement::GdsArrayRef(GdsArrayRef {
                    name: ar.id,
                    xy: [
                        to_gds_point(ar.start, scale),
                        to_gds_point(ar.col_end, scale),
                        to_gds_point(ar.row_end, scale),
                    ],
                    cols: ar.rows,
                    rows: ar.cols,
                    strans: ar.strans,
                    ..Default::default()
                })),
            }
        }
        new_cell
    }
}

pub(crate) trait ToGds21Library {
    fn to_gds21_library(self) -> gds21::GdsLibrary;
}

impl<T> ToGds21Library for super::DgirLibrary<T, Length<Absolute, T>>
where
    T: Num + FromPrimitive,
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
                let database_unit = self.units.database;
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
                        structs.push(first_cell.to_gds21_struct(database_unit));
                        for s in dependencies {
                            structs.push(
                                Rc::try_unwrap(s)
                                    .expect("all rc points should be in set, which makes it unique")
                                    .to_gds21_struct(database_unit),
                            );
                        }
                        structs
                    }
                }
            },
            units: gds21::GdsUnits::new(
                (self.units.database / self.units.user).to_f64().unwrap(),
                (self.units.database / Length::new_absolute::<crate::units::Meter>(T::one()))
                    .to_f64()
                    .unwrap(),
            ),
            ..Default::default()
        }
    }
}

impl<T> ToGds21Library for super::DgirLibrary<T, Length<Relative, T>>
where
    T: Num + FromPrimitive,
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
                (self.units.database / self.units.user).to_f64().unwrap(),
                (self.units.database / Length::new_absolute::<crate::units::Meter>(T::one()))
                    .to_f64()
                    .unwrap(),
            ),
            ..Default::default()
        }
    }
}
