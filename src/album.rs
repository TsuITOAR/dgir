use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use num::{FromPrimitive, ToPrimitive};

use crate::{
    draw::{Brush, Coordinate},
    paint::ColorDrawing,
};

pub struct Path<T: Brush> {
    pub path: ColorDrawing<T>,
    pub width: T,
}
impl<T: Brush + 'static> Path<T> {
    pub fn to_painting(self) -> Painting<T>
    where
        T: Clone,
    {
        Painting::Path(Path {
            path: self.path,
            width: self.width,
        })
    }
}

pub struct Polygon<T: Brush> {
    pub polygon: ColorDrawing<T>,
}
impl<T: Brush + 'static> Polygon<T> {
    pub fn to_painting(self) -> Painting<T>
    where
        T: Clone,
    {
        Painting::Polygon(Polygon {
            polygon: self.polygon,
        })
    }
}

pub struct Ref<T: Brush> {
    pub(crate) decorator: Option<gds21::GdsStrans>,
    pub(crate) position: Coordinate<T>,
    pub(crate) reference: String,
    pub(crate) dependencies: BTreeSet<Rc<Album<T>>>,
}

impl<T: Brush> From<Album<T>> for Ref<T> {
    fn from(album: Album<T>) -> Self {
        let mut dependencies = BTreeSet::new();
        for painting in album.paintings.iter() {
            match painting {
                Painting::Ref(r) => dependencies.append(&mut r.dependencies.clone()),
                _ => (),
            }
        }
        let album = Rc::new(album);
        if !dependencies.insert(album.clone()) {
            eprint!("circular references or duplicated names: {}", album.name);
        }
        Self {
            decorator: None,
            position: Coordinate::default(),
            reference: album.name.clone(),
            dependencies,
        }
    }
}

impl<T: Brush> Ref<T> {
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
pub enum Painting<T: Brush> {
    Path(Path<T>),
    Polygon(Polygon<T>),
    Ref(Ref<T>),
}

/* impl<'a, T: Clone + Brush + Convert<U> + 'static, U: Brush> Convert<Painting<'a, U>>
    for Painting<'a, T>
{
    fn convert(self) -> Painting<'a, U> {
        match self {
            Painting::Path(path) => Painting::Path(Path {
                path: path.path.convert(),
                width: path.width.convert(),
            }),
            Painting::Polygon(polygon) => Painting::Polygon(Polygon {
                polygon: polygon.polygon.convert(),
            }),
            Painting::Ref(r) => Painting::Ref(Ref {
                position: r.position.convert(),
                decorator: r.decorator,
                reference: r.reference.convert(),
            }),
        }
    }
}
 */

pub struct Album<T: Brush> {
    pub name: String,
    pub(crate) paintings: Vec<Painting<T>>,
}

impl<T: Brush> Album<T> {
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
    pub fn as_ref(self) -> Ref<T> {
        self.into()
    }
    pub fn get_dependencies(&self) -> BTreeSet<Rc<Album<T>>> {
        let mut dependencies = BTreeSet::new();
        for painting in self.paintings.iter() {
            match painting {
                Painting::Ref(r) => {
                    dependencies.append(&mut r.dependencies.clone());
                }
                _ => (),
            }
        }
        dependencies
    }

    pub fn to_cell(
        self,
        database_unit: T,
    ) -> gds21::GdsStruct
    where
        T: Clone + Copy,
        <T as Brush>::Basic: ToPrimitive + FromPrimitive,
    {
        use gds21::*;
        let mut new_cell = GdsStruct::new(self.name);
        for painting in self.paintings {
            new_cell.elems.push(match painting {
                Painting::Path(p) => GdsElement::GdsPath(GdsPath {
                    layer: p.path.color.layer,
                    datatype: p.path.color.datatype,
                    xy: p.path.drawing.to_xy(database_unit),
                    width: (p.width / database_unit).to_i32(),
                    ..Default::default()
                }),
                Painting::Polygon(p) => GdsElement::GdsBoundary(GdsBoundary {
                    layer: p.polygon.color.layer,
                    datatype: p.polygon.color.datatype,
                    xy: p.polygon.drawing.to_xy(database_unit),
                    ..Default::default()
                }),
                Painting::Ref(r) => {
                    GdsElement::GdsStructRef(GdsStructRef {
                        name: r.reference,
                        xy: r
                            .position
                            .into_iter()
                            .map(|x| (x / database_unit).to_i32().unwrap())
                            .collect(),
                        strans: r.decorator,
                        ..Default::default()
                    })
                }
            })
        }
        new_cell
    }
}

impl<T: Brush> PartialEq for Album<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
impl<T: Brush> Eq for Album<T> {}

impl<T: Brush> PartialOrd for Album<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
impl<T: Brush> Ord for Album<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<T: Brush> Deref for Album<T> {
    type Target = Vec<Painting<T>>;
    fn deref(&self) -> &Self::Target {
        &self.paintings
    }
}

impl<T: Brush> DerefMut for Album<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.paintings
    }
}

/* impl<'a, T: Clone + Brush + Convert<U> + 'static, U: Brush> Convert<Album<'a, U>> for Album<'a, T> {
    fn convert(self) -> Album<'a, U> {
        Album::<'a, U> {
            name: self.name,
            //TO-DO:this consumes paintings, break the share between structure, change this to a decorator after the original output
            paintings: self.paintings.into_iter().map(|x| x.convert()).collect(),
        }
    }
} */
