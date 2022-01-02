use crate::Quantity;

use self::groups::{Compound, Group};

use super::coordinate::Coordinate;

#[cfg(feature = "rayon")]
pub mod par_iter;

pub mod groups;
pub mod iter;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Curve<C> {
    pub(crate) curve: C,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Area<A> {
    pub(crate) area: A,
}

pub trait IntoCurve {
    type Q: Quantity;
    type Curve: IntoIterator<Item = Coordinate<Self::Q>>;
    fn into_curve(self) -> Curve<Self::Curve>;
}

pub trait IntoArea {
    type Q: Quantity;
    type Area: IntoIterator<Item = Coordinate<Self::Q>>;
    fn into_area(self) -> Area<Self::Area>;
}

impl<A, Q> IntoArea for Area<A>
where
    Q: Quantity,
    A: IntoIterator<Item = Coordinate<Q>>,
{
    type Q = Q;
    type Area = A;
    fn into_area(self) -> Area<Self::Area> {
        self
    }
}

pub trait Sweep<Q: Quantity> {
    type Output: IntoIterator<Item = Coordinate<Q>>;
    fn sweep(self, range: (Q, Q)) -> Area<Self::Output>;
}

pub trait Bias<Q> {
    fn bias(&mut self, b: Q);
}

pub trait Split<P>: Sized {
    fn split(self, pos: P) -> (Self, Self);
}

pub trait SplitHalf<P>: Split<P> {
    fn split_half(self) -> (Self, Self);
}

impl<Q, T1, T2> Bias<Q> for Compound<T1, T2>
where
    Q: Quantity,
    T1: Bias<Q>,
    T2: Bias<Q>,
{
    fn bias(&mut self, b: Q) {
        self.0.bias(b.clone());
        self.1.bias(b);
    }
}

impl<Q, T> Bias<Q> for Group<T>
where
    Q: Quantity,
    T: Bias<Q>,
{
    fn bias(&mut self, b: Q) {
        for t in self.0.iter_mut() {
            t.bias(b.clone());
        }
    }
}

impl<P, T1, T2> Split<P> for Compound<T1, T2>
where
    P: Clone,
    T1: Split<P>,
    T2: Split<P>,
{
    fn split(self, pos: P) -> (Self, Self) {
        let t1 = self.0.split(pos.clone());
        let t2 = self.1.split(pos);
        (Self(t1.0, t2.1), Self(t1.1, t2.0))
    }
}

impl<P, T> Split<P> for Group<T>
where
    P: Clone,
    T: Split<P>,
{
    fn split(self, pos: P) -> (Self, Self) {
        let (left, right): (Vec<_>, Vec<_>) = self
            .0
            .into_iter()
            .map(move |x| x.split(pos.clone()))
            .unzip();
        (Group(left), Group(right))
    }
}

macro_rules! wrapper_impl {
    (
        $wrapper:ident<$wgen:ident$(:$($wbound:ident)*)?$(,)*>,
        $field:ident,
        $trait:ident$(<$($tgen:ident$(:$($tbound:ident)*)?),+$(,)*>)?,
        $fun:ident(self$(,$arg:ident:$arg_type:ty)*$(,)*)$(->$ret:ty)?
        $(,associate $asso_type:ident)*
        $(,impl_fn $temp:ident $im:expr)?
        $(,)*
    ) => {
            impl<$wgen:$trait$(<$($tgen),*>)? $(+$($wbound:ident)+*)? $(,$($tgen$(:$($tbound)*)?),+)?> $trait$(<$($tgen),*>)? for $wrapper<$wgen>
            {
                $(type $asso_type=T::$asso_type;)*
                fn $fun(self$(,$arg:$arg_type)*)$(->$ret)?{
                    wrapper_impl!(@get_impl self.$field.$fun($($arg),*)$(,$temp,$im)?)
                }
            }
        };
        (
        $wrapper:ident<$wgen:ident$(:$($wbound:ident)*)?$(,)*>,
        $field:ident,
        $trait:ident$(<$($tgen:ident$(:$($tbound:ident)*)?),+$(,)*>)?,
        $fun:ident(&mut self$(,$arg:ident:$arg_type:ty)*$(,)*)$(->$ret:ty)?
        $(,associate $asso_type:ident)*
        $(,impl_fn $temp:ident $im:expr)?
        $(,)*
    ) => {
            impl<$wgen:$trait$(<$($tgen),*>)? $(+$($wbound:ident)+*)? $(,$($tgen$(:$($tbound)*)?),+)?> $trait$(<$($tgen),*>)? for $wrapper<$wgen>
            {
                $(type $asso_type=T::$asso_type;)*
                fn $fun(&mut self$(,$arg:$arg_type)*)$(->$ret)?{
                    wrapper_impl!(@get_impl self.$field.$fun($($arg),*)$(,$temp,$im)?)
                }
            }
        };
        (
        $wrapper:ident<$wgen:ident$(:$($wbound:ident)*)?$(,)*>,
        $field:ident,
        $trait:ident$(<$($tgen:ident$(:$($tbound:ident)*)?),+$(,)*>)?,
        $fun:ident(&self$(,$arg:ident:$arg_type:ty)*$(,)*)$(->$ret:ty)?
        $(,associate $asso_type:ident)*
        $(,impl_fn $temp:ident $im:expr)?
        $(,)*
    ) => {
            impl<$wgen:$trait$(<$($tgen),*>)? $(+$($wbound:ident)+*)? $(,$($tgen$(:$($tbound)*)?),+)?> $trait$(<$($tgen),*>)? for $wrapper<$wgen>
            {
                $(type $asso_type=T::$asso_type;)*
                fn $fun(&self$(,$arg:$arg_type)*)$(->$ret)?{
                    wrapper_impl!(@get_impl self.$field.$fun($($arg),*)$(,$temp,$im)?)
                }
            }
        };
        (
        $wrapper:ident<$wgen:ident$(:$($wbound:ident)*)?$(,)*>,
        $field:ident,
        $trait:ident$(<$($tgen:ident$(:$($tbound:ident)*)?),+$(,)*>)?,
        $fun:ident($($arg:ident:$arg_type:ty),*$(,)*)$(->$ret:ty)?
        $(,associate $asso_type:ident)*
        $(,impl_fn $temp:ident $im:expr)?
        $(,)*
    ) => {
            impl<$wgen:$trait$(<$($tgen),*>)? $(+$($wbound:ident)+*)? $(,$($tgen$(:$($tbound)*)?),+)?> $trait$(<$($tgen),*>)? for $wrapper<$wgen>
            {
                $(type $asso_type=T::$asso_type;)*
                fn $fun($($arg:$arg_type,)*)$(->$ret)?{
                    wrapper_impl!(@get_impl self.$field::$fun($($arg),*)$(,$temp,$im)?)
                }
            }
        };
    (
        @get_impl $origin:expr,$temp:ident,$im:expr$(,)*
    ) =>{
            {
                let $temp=$origin;
                $im
            }
        };
    (
        @get_impl $origin:expr$(,)*
    ) =>{
            $origin
        };
}

wrapper_impl!(Curve<T>,curve,Split<P>,split(self,pos:P)->(Self,Self),impl_fn a (Curve{curve:a.0},Curve{curve:a.1}));
wrapper_impl!(Area<T>,area,Split<P>,split(self,pos:P)->(Self,Self),impl_fn a (Area{area:a.0},Area{area:a.1}));
wrapper_impl!(Curve<T>,curve,SplitHalf<P>,split_half(self)->(Self,Self),impl_fn a (Curve{curve:a.0},Curve{curve:a.1}));
wrapper_impl!(Area<T>,area,SplitHalf<P>,split_half(self)->(Self,Self),impl_fn a (Area{area:a.0},Area{area:a.1}));
wrapper_impl!(Curve<T>, curve, Bias<Q: Quantity>, bias(&mut self, b: Q));
wrapper_impl!(Area<T>, area, Bias<Q: Quantity>, bias(&mut self, b: Q));
