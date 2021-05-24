mod elements;

pub struct Drawer<In, Out> {
    list: Box<dyn Iterator<Item = In>>,
    x: Box<dyn FnMut(In) -> Out>,
    y: Box<dyn FnMut(In) -> Out>,
}

impl<'a, In: 'a, Out: 'a> Drawer<In, Out> {
    fn draw(self) -> Drawing<'a, Out> {
        Drawing::Iter(Box::new(self.list.map(|p| [(self.x)(p), (self.y)(p)])))
    }
    fn decorate(&mut self, decorator: Box<dyn FnMut(In) -> In>) -> &mut Self {
        self.list = self.list.map(|p| decorator(p));
        self
    }
}
enum Drawing<'a, T> {
    Iter(Box<dyn Iterator<Item = [T; 2]> + 'a>),
    Points(Vec<[T; 2]>),
}
