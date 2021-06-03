pub mod album;
pub mod draw;
pub mod paint;
pub mod units;

use album::Album;
use draw::Brush;
use num::{FromPrimitive, ToPrimitive};
use std::{
    collections::BTreeSet,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use units::AbsoluteLength;

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
    pub fn to_gds(self, user_unit: T, database_unit: T) -> gds21::GdsLibrary
    where
        T: Brush + Clone + Copy,
        <T as Brush>::Basic: ToPrimitive + FromPrimitive,
    {
        use gds21::*;
        let mut lib = GdsLibrary::new(self.name);
        let mut dependencies = BTreeSet::new();
        lib.units = GdsUnits::new(
            (user_unit / database_unit).to_f64().unwrap(),
            (database_unit / <T as Brush>::from(1.)).to_f64().unwrap(),
        );
        for album in self.albums {
            dependencies.append(&mut album.get_dependencies());
            dependencies.insert(Rc::new(album));
        }
        for mut album in dependencies {
            lib.structs.push(
                mem::replace(Rc::get_mut(&mut album).unwrap(), Album::new("_0"))
                    .to_cell(database_unit),
            );
        }
        lib
    }
}

pub type Lib<'a> = Library<AbsoluteLength<f64>>;
pub type Cell<'a> = Album<AbsoluteLength<f64>>;

pub const NANOMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e-3,
    marker: PhantomData,
};

pub const MICROMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e0,
    marker: PhantomData,
};

pub const MILLIMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e3,
    marker: PhantomData,
};

pub const CENTIMETER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e4,
    marker: PhantomData,
};

pub const METER: AbsoluteLength<f64> = AbsoluteLength::<f64> {
    value: 1e6,
    marker: PhantomData,
};
