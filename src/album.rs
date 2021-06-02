use std::ops::{Deref, DerefMut};

use crate::{
    draw::{Brush, Convert},
    paint::ColorDrawing,
    units::{Length, Micrometer},
};

pub struct Path<T: Brush> {
    pub path: ColorDrawing<T>,
    pub width: T,
}
impl<'a, T: Brush + 'static> Path<T> {
    pub fn to_painting<U: Brush>(self) -> Painting<'a, U>
    where
        T: Clone + Convert<U>,
    {
        Painting::Path(Path::<U> {
            path: self.path.convert(),
            width: self.width.convert(),
        })
    }
}

pub struct Polygon<T: Brush> {
    pub polygon: ColorDrawing<T>,
}
impl<'a, T: Brush + 'static> Polygon<T> {
    pub fn to_painting<U: Brush>(self) -> Painting<'a, U>
    where
        T: Clone + Convert<U>,
    {
        Painting::Polygon(Polygon::<U> {
            polygon: self.polygon.convert(),
        })
    }
}
#[derive(Clone, Default, Debug)]
pub struct Decorator {
    trans: gds21::GdsStrans,
}

pub struct Ref<'a, T: Brush> {
    decorator: Decorator,
    reference: &'a Album<'a, T>,
}

impl<'a, T: Brush> From<&'a Album<'a, T>> for Ref<'a, T> {
    fn from(album: &'a Album<T>) -> Self {
        Self {
            decorator: Decorator::default(),
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

impl<'a, T: Clone + Brush + Convert<U> + 'static, U: Brush> Convert<Painting<'a, U>>
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
            Painting::Ref(r) => {}
        }
    }
}

pub struct Album<'a, T: Brush = Length<Micrometer, f64>> {
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
impl<'a, T: Clone + Brush + Convert<U> + 'static, U: Brush> Convert<Album<'a, U>> for Album<'a, T> {
    fn convert(self) -> Album<'a, U> {
        Album::<'a, U> {
            name: self.name,
            //TO-DO:this consumes paintings, break the share between structure, change this to a decorator after the original output
            paintings: self.paintings.into_iter().map(|x| x.convert()).collect(),
        }
    }
}
