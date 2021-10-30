use gds21::GdsPoint;

pub mod album;
pub mod color;
pub mod draw;
pub mod units;
const MAX_POINTS_NUM: usize = 8191;
/*
use gds21::GdsPoint;
use num::{FromPrimitive, ToPrimitive};
use std::{
    collections::BTreeSet,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use units::AbsoluteLength;


pub struct Library<T: Distance> {
    name: String,
    albums: Vec<Album<T>>,
}
impl<T: Distance> Deref for Library<T> {
    type Target = Vec<Album<T>>;
    fn deref(&self) -> &Self::Target {
        &self.albums
    }
}

impl<T: Distance> DerefMut for Library<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.albums
    }
}
impl<T: Distance> Library<T> {
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
        T: Distance + Clone + Copy,
        <T as Distance>::Basic: ToPrimitive + FromPrimitive,
    {
        use gds21::*;
        let mut lib = GdsLibrary::new(self.name);
        let mut dependencies = BTreeSet::new();
        lib.units = GdsUnits::new(
            (user_unit / database_unit).to_f64().unwrap(),
            (database_unit / <T as Distance>::from(1.))
                .to_f64()
                .unwrap(),
        );
        for mut album in self.albums {
            dependencies.append(&mut album.get_dependencies());
            dependencies.insert(Rc::new(album));
        }
        for mut album in dependencies {
            lib.structs.push(
                mem::replace(Rc::get_mut(&mut album).unwrap(), Album::new(String::new()))
                    .to_cell(database_unit),
            );
        }
        lib
    }
}



pub type Lib = Library<AbsoluteLength<f64>>;
pub type Cell = Album<AbsoluteLength<f64>>;

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
 */

fn points_num_check(points: &Vec<GdsPoint>) -> bool {
    if points.len() > MAX_POINTS_NUM {
        eprint!(
            "points number({}) exceeds limit({})",
            points.len(),
            MAX_POINTS_NUM
        );
        return false;
    }
    return true;
}

fn close_curve(points: &mut Vec<GdsPoint>) -> bool {
    if points.len() >= 1
        && points[points.len() - 1] != points[2]
        && points[points.len() - 2] != points[1]
    {
        eprint!(
            "curve not closed, start at ({}, {}), end at ({}, {})",
            points[0].x,
            points[0].y,
            points.last().unwrap().x,
            points.last().unwrap().y
        );
        false
    } else {
        true
    }
}
