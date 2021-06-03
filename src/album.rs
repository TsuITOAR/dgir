use std::ops::{Deref, DerefMut};

use crate::{
    draw::{Brush, Coordinate},
    paint::ColorDrawing,
};

pub struct Path<T: Brush> {
    pub path: ColorDrawing<T>,
    pub width: T,
}
impl<'a, T: Brush + 'static> Path<T> {
    pub fn to_painting(self) -> Painting<'a, T>
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
impl<'a, T: Brush + 'static> Polygon<T> {
    pub fn to_painting(self) -> Painting<'a, T>
    where
        T: Clone,
    {
        Painting::Polygon(Polygon {
            polygon: self.polygon,
        })
    }
}
#[derive(Clone, Default, Debug)]
pub struct Decorator {
    pub(crate) trans: gds21::GdsStrans,
}

pub struct Ref<'a, T: Brush> {
    pub(crate) decorator: Decorator,
    pub(crate) position: Coordinate<T>,
    pub(crate) reference: &'a Album<'a, T>,
}

impl<'a, T: Brush> From<&'a Album<'a, T>> for Ref<'a, T> {
    fn from(album: &'a Album<T>) -> Self {
        Self {
            decorator: Decorator::default(),
            position: Coordinate::default(),
            reference: album,
        }
    }
}

impl<'a, T: Brush> Ref<'a, T> {
    pub fn new(album: &'a Album<T>) -> Self {
        Self::from(album)
    }
    pub fn set_decorator(&mut self, decorator: Decorator) -> &mut Self {
        self.decorator = decorator;
        self
    }
    pub fn decorator_mut(&mut self) -> &mut Decorator {
        &mut self.decorator
    }
}
pub enum Painting<'a, T: Brush> {
    Path(Path<T>),
    Polygon(Polygon<T>),
    Ref(Ref<'a, T>),
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
pub struct Album<'a, T: Brush> {
    pub name: String,
    pub(crate) paintings: Vec<Painting<'a, T>>,
}

impl<'a, T: Brush> Album<'a, T> {
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
}

impl<'a, T: Brush> Deref for Album<'a, T> {
    type Target = Vec<Painting<'a, T>>;
    fn deref(&self) -> &Self::Target {
        &self.paintings
    }
}

impl<'a, T: Brush> DerefMut for Album<'a, T> {
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
