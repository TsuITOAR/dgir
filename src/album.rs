use crate::{draw::Brush, paint::ColorfulDrawing};

struct Path<T:Brush>{
    path:ColorfulDrawing<T>,
    width:<T as Brush>::Basic
}

struct Polygon<T:Brush>{
    polygon:ColorfulDrawing<T>
}

struct Ref<T>{
    reference:Rc<Album<T>>
}
pub enum Painting{
    Path()
}

pub struct Album<T>{

}