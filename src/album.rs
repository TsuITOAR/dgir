use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    draw::{Brush, Convert},
    paint::ColorDrawing,
    units::{Length, Micrometer},
};

pub struct Path<T: Brush> {
    pub path: ColorDrawing<T>,
    pub width: T,
}
impl<T: Brush + 'static> Path<T> {
    pub fn to_painting<U: Brush>(self) -> Painting<U>
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
impl<T: Brush + 'static> Polygon<T> {
    pub fn to_painting<U: Brush>(self) -> Painting<U>
    where
        T: Clone + Convert<U>,
    {
        Painting::Polygon(Polygon::<U> {
            polygon: self.polygon.convert(),
        })
    }
}
/* struct Ref<T: Brush> {
    reference: Rc<Album<T>>,
}

impl<T: Brush, U: Brush> AsRef<Ref<U>> for Ref<T> {}
impl<T: Brush + 'static> Ref<T> {
    pub fn to_painting<U: Brush>(self) -> Painting<U>
    where
        T: Clone + Convert<U>,
    {
        Painting::Ref(Ref::<U> {
            reference: Rc::new(self.reference.convert()),
        })
    }
} */
pub enum Painting<T: Brush> {
    Path(Path<T>),
    Polygon(Polygon<T>),
    /* Ref(Ref<T>), */
}

impl<T: Clone + Brush + Convert<U> + 'static, U: Brush> Convert<Painting<U>> for Painting<T> {
    fn convert(self) -> Painting<U> {
        match self {
            Painting::Path(path) => Painting::Path(Path {
                path: path.path.convert(),
                width: path.width.convert(),
            }),
            Painting::Polygon(polygon) => Painting::Polygon(Polygon {
                polygon: polygon.polygon.convert(),
            }),
            /* Painting::Ref(r) => Painting::Ref(Ref {
                reference: Rc::new(r.reference.convert()),
            }), */
        }
    }
}

pub struct Album<T: Brush = Length<Micrometer, f64>> {
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
impl<T: Clone + Brush + Convert<U> + 'static, U: Brush> Convert<Album<U>> for Album<T> {
    fn convert(self) -> Album<U> {
        Album::<U> {
            name: self.name,
            //TO-DO:this consumes paintings, break the share between structure, change this to a decorator after the original output
            paintings: self.paintings.into_iter().map(|x| x.convert()).collect(),
        }
    }
}
