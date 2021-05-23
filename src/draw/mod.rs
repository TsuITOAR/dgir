mod elements;

enum Drawer<In, Out> {
    CurveEquation(Box<dyn Fn(Out) -> Out>),
    ParametricEquation((Box<dyn Fn(In) -> Out>, Box<dyn Fn(In) -> Out>)),
    DifferentialEquation(Box<dyn FnMut(Out) -> Out>),
    DifferentialParametricEquation((Box<dyn FnMut(In) -> Out>, Box<dyn FnMut(In) -> Out>)),
}
enum Drawing<'a, T: 'a, S> {
    Iter(Box<dyn Iterator<Item = T> + 'a>),
    Points(Vec<[T; 2]>),
    Drawer(Drawer<S, T>),
}
