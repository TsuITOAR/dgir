use std::{ops::{Index, IndexMut}, rc::Rc};

use crate::{
    draw::{Brush, Convert},
    paint::ColorDrawing,
};

struct Path<T: Brush> {
    path: ColorDrawing<T>,
    width: T,
}

struct Polygon<T: Brush> {
    polygon: ColorDrawing<T>,
}

struct Ref<T: Brush> {
    reference: Rc<Album<T>>,
}
pub enum Painting<T: Brush> {
    Path(Path<T>),
    Polygon(Polygon<T>),
    Ref(Ref<T>),
}

impl<T: Brush + Convert<U> + 'static, U: Brush> Convert<Painting<U>> for Painting<T> {
    fn convert(self) -> Painting<U> {
        match self {
            Painting::Path(path) => Painting::Path(Path {
                path: path.path.convert(),
                width: path.width.convert(),
            }),
            Painting::Polygon(polygon) => Painting::Polygon(Polygon {
                polygon: polygon.polygon.convert(),
            }),
            Painting::Ref(r) => Painting::Ref(Ref {
                reference: Rc::new(r.reference.convert()),
            }),
        }
    }
}
impl<T: Brush + Convert<U> + 'static, U: Brush> Convert<Album<U>> for Album<T> {
    fn convert(self) -> Album<U> {
        Album::<U> {
            name: self.name,
            //TO-DO:this consumes paintings, break the share between structure, change this to a decorator after the original output
            paintings: self.paintings.into_iter().map(|x| x.convert()).collect(),
        }
    }
}

pub struct Album<T: Brush> {
    name: String,
    paintings: Vec<Painting<T>>,
}

impl<T: Brush> Album<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            paintings: Vec::new(),
        }
    }
    pub fn rename(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn push<U: Convert<T> + Brush + 'static>(&mut self, painting: Painting<U>) -> &mut Self {
        self.paintings.push(painting.convert());
        self
    }
}
impl<T:Brush> Index<usize> for Album<T>{
    type Output = Painting<T>;
    fn index(&self, index: usize) -> &Self::Output {
        self.paintings.index(index)
    }
}
impl<T:Brush> IndexMut<usize> for Album<T>{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.paintings.index_mut(index)
    }
}