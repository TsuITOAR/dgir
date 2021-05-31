pub mod album;
pub mod draw;
pub mod paint;
pub mod units;

use album::Album;
use draw::{Brush, Convert};
use num::ToPrimitive;
use std::ops::{Deref, DerefMut};
use units::Micrometer;

use crate::{
    album::Painting,
    units::{Length, Meter},
};
pub struct Library<T: Brush> {
    name: String,
    albums: Vec<Album<T>>,
}
impl<T: Brush> Deref for Library<T> {
    type Target = Vec<Album<T>>;
    fn deref(&self) -> &Self::Target {
        &self.albums
    }
}

impl<T: Brush> DerefMut for Library<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.albums
    }
}
impl<T: Brush> Library<T> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            albums: Vec::new(),
        }
    }
    pub fn rename(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn to_gds(self, user_unit: f64, database_unit: f64) -> gds21::GdsLibrary
    where
        Length<Meter>: Convert<T>,
        T: Brush + Clone,
    {
        use gds21::*;
        let mut lib = GdsLibrary::new(self.name);
        lib.units = GdsUnits::new(user_unit / database_unit, database_unit);
        let database_unit = Length::<Meter>::new(database_unit);
        for album in self.albums {
            lib.structs.push({
                let mut new_cell = GdsStruct::new(album.name);
                for painting in album.paintings {
                    new_cell.elems.push(match painting {
                        Painting::Path(p) => GdsElement::GdsPath(GdsPath {
                            layer: p.path.color.layer,
                            datatype: p.path.color.datatype,
                            xy: p.path.drawing.to_xy(database_unit),
                            width: (p.width / database_unit.convert()).to_i32(),
                            ..Default::default()
                        }),
                        Painting::Polygon(p) => GdsElement::GdsBoundary(GdsBoundary {
                            layer: p.polygon.color.layer,
                            datatype: p.polygon.color.datatype,
                            xy: p.polygon.drawing.to_xy(database_unit),
                            ..Default::default()
                        }),
                    })
                }
                new_cell
            })
        }
        lib
    }
}

pub type Lib = Library<Length<Micrometer, f64>>;
pub type Alb = Album<Length<Micrometer, f64>>;
