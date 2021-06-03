pub mod album;
pub mod draw;
pub mod paint;
pub mod units;

use album::Album;
use draw::Brush;
use num::{FromPrimitive, ToPrimitive};
use std::ops::{Deref, DerefMut};
use units::Micrometer;

use crate::{
    album::Painting,
    units::{MakeLength, Meter},
};
pub struct Library<'a, T: Brush> {
    name: String,
    albums: Vec<Album<'a, T>>,
}
impl<'a, T: Brush> Deref for Library<'a, T> {
    type Target = Vec<Album<'a, T>>;
    fn deref(&self) -> &Self::Target {
        &self.albums
    }
}

impl<'a, T: Brush> DerefMut for Library<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.albums
    }
}
impl<'a, T: Brush> Library<'a, T> {
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
    pub fn to_gds(self, user_unit: T, database_unit: T) -> gds21::GdsLibrary
    where
        T: Brush + Clone,
        <T as Brush>::Basic: ToPrimitive + FromPrimitive,
    {
        use gds21::*;
        let mut lib = GdsLibrary::new(self.name);
        lib.units = GdsUnits::new(
            (user_unit / database_unit).to_f64().unwrap(),
            (database_unit / <T as Brush>::from(1.)).to_f64().unwrap(),
        );
        let database_length =
            MakeLength::<Meter, <T as Brush>::Basic>::new_absolute(database_unit);
        for album in self.albums {
            lib.structs.push({
                let mut new_cell = GdsStruct::new(album.name);
                for painting in album.paintings {
                    new_cell.elems.push(match painting {
                        Painting::Path(p) => GdsElement::GdsPath(GdsPath {
                            layer: p.path.color.layer,
                            datatype: p.path.color.datatype,
                            xy: p.path.drawing.to_xy(database_length),
                            width: (p.width / database_length).to_i32(),
                            ..Default::default()
                        }),
                        Painting::Polygon(p) => GdsElement::GdsBoundary(GdsBoundary {
                            layer: p.polygon.color.layer,
                            datatype: p.polygon.color.datatype,
                            xy: p.polygon.drawing.to_xy(database_unit),
                            ..Default::default()
                        }),
                        Painting::Ref(r) => GdsElement::GdsStructRef(GdsStructRef {
                            name: r.reference.name,
                            xy: r.position.into_iter().map(|x| (x / database_unit).to_i32()),
                            strans: r.decorator.trans,
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

pub type Lib<'a> = Library<'a, MakeLength<Micrometer, f64>>;
pub type Alb<'a> = Album<'a, MakeLength<Micrometer, f64>>;
