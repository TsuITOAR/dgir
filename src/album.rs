use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use num::{FromPrimitive, ToPrimitive};

use crate::{
    close_curve,
    color::LayerData,
    draw::{Coordinate, Distance, Drawing},
    points_num_check,
};

pub struct Path<T: Distance> {
    pub coordinates: Drawing<T>,
    pub color: LayerData,
    pub width: Option<T>,
}
impl<T: Distance + 'static> Path<T> {
    pub fn to_painting(self) -> Painting<T>
    where
        T: Clone,
    {
        Painting::Path(self)
    }
}

pub struct Polygon<T: Distance> {
    pub coordinates: Drawing<T>,
    pub color: LayerData,
}
impl<T: Distance + 'static> Polygon<T> {
    pub fn to_painting(self) -> Painting<T>
    where
        T: Clone,
    {
        Painting::Polygon(self)
    }
}

pub struct Ref<T: Distance> {
    pub(crate) decorator: Option<gds21::GdsStrans>,
    pub(crate) position: Coordinate<T>,
    pub(crate) reference: String,
    pub(crate) dependencies: Option<BTreeSet<Rc<Album<T>>>>,
}

impl<T: Distance> From<Album<T>> for Ref<T> {
    fn from(mut album: Album<T>) -> Self {
        let mut dependencies = BTreeSet::new();
        for painting in album.paintings.iter_mut() {
            match painting {
                Painting::Ref(Ref {
                    dependencies: Some(ref mut d),
                    ..
                }) => dependencies.append(d),
                _ => (),
            }
        }
        let album = Rc::new(album);
        if !dependencies.insert(album.clone()) {
            eprint!("circular references or duplicated names: {}", album.name);
        }
        Self {
            decorator: None,
            position: Coordinate::from([T::zero(), T::zero()]),
            reference: album.name.clone(),
            dependencies: Some(dependencies),
        }
    }
}

impl<T: Distance> Ref<T> {
    pub fn new(album: Album<T>) -> Self {
        Self::from(album)
    }
    pub fn set_position(&mut self, position: Coordinate<T>) -> &mut Self {
        self.position = position;
        self
    }
    pub fn set_decorator(&mut self, strans: gds21::GdsStrans) -> &mut Self {
        self.decorator = Some(strans);
        self
    }
    pub fn decorator_mut(&mut self) -> &mut Option<gds21::GdsStrans> {
        &mut self.decorator
    }
}
pub enum Painting<T: Distance> {
    Path(Path<T>),
    Polygon(Polygon<T>),
    Ref(Ref<T>),
}

impl<T: Distance> From<Path<T>> for Painting<T> {
    fn from(p: Path<T>) -> Self {
        Painting::Path(p)
    }
}

impl<T: Distance> From<Polygon<T>> for Painting<T> {
    fn from(p: Polygon<T>) -> Self {
        Painting::Polygon(p)
    }
}
impl<T: Distance> From<Ref<T>> for Painting<T> {
    fn from(r: Ref<T>) -> Self {
        Painting::Ref(r)
    }
}
pub struct Album<T: Distance> {
    pub name: String,
    pub(crate) paintings: Vec<Painting<T>>,
}

impl<T: Distance> Album<T> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            paintings: Vec::new(),
        }
    }
    pub fn rename(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn insert<U: Into<Painting<T>>>(&mut self, painting: U) -> &mut Self {
        self.push(painting.into());
        self
    }

    pub fn as_ref(self) -> Ref<T> {
        self.into()
    }
    pub fn get_dependencies(&mut self) -> BTreeSet<Rc<Album<T>>> {
        let mut dependencies = BTreeSet::new();
        for painting in self.paintings.iter_mut() {
            match painting {
                Painting::Ref(Ref {
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

    pub fn to_cell(self, database_unit: T) -> gds21::GdsStruct
    where
        T: Clone + Copy,
        <T as Distance>::Basic: ToPrimitive + FromPrimitive,
    {
        use gds21::*;
        let mut new_cell = GdsStruct::new(self.name);
        for painting in self.paintings {
            new_cell.elems.push(match painting {
                Painting::Path(p) => GdsElement::GdsPath({
                    let xy = p.coordinates.to_xy(database_unit);
                    points_num_check(&xy);
                    GdsPath {
                        layer: p.color.layer,
                        datatype: p.color.datatype,
                        xy,
                        width: match p.width {
                            Some(l) => (l / database_unit).to_i32(),
                            None => None,
                        },
                        ..Default::default()
                    }
                }),
                Painting::Polygon(p) => GdsElement::GdsBoundary({
                    let mut xy = p.coordinates.to_xy(database_unit);
                    close_curve(&mut xy);
                    points_num_check(&xy);
                    GdsBoundary {
                        layer: p.color.layer,
                        datatype: p.color.datatype,
                        xy,
                        ..Default::default()
                    }
                }),
                Painting::Ref(r) => GdsElement::GdsStructRef(GdsStructRef {
                    name: r.reference,
                    xy: r
                        .position
                        .into_iter()
                        .map(|x| (x / database_unit).to_i32().unwrap())
                        .collect(),
                    strans: r.decorator,
                    ..Default::default()
                }),
            })
        }
        new_cell
    }
}

impl<T: Distance> PartialEq for Album<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
impl<T: Distance> Eq for Album<T> {}

impl<T: Distance> PartialOrd for Album<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
impl<T: Distance> Ord for Album<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<T: Distance> Deref for Album<T> {
    type Target = Vec<Painting<T>>;
    fn deref(&self) -> &Self::Target {
        &self.paintings
    }
}

impl<T: Distance> DerefMut for Album<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.paintings
    }
}
