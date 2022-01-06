use std::{collections::BTreeSet, fmt::Debug, rc::Rc};

use gds21::GdsStrans;
use num::{traits::FloatConst, FromPrimitive, ToPrimitive};

use crate::{
    color::LayerData,
    draw::coordinate::Coordinate,
    units::{Absolute, AbsoluteLength, Angle, Length, LengthType, Relative},
    Num, Quantity,
};

use self::togds::ToGds21Library;

mod togds;

//const DISPLAY_POINTS_NUM: usize = 20;
type Result<T> = gds21::GdsResult<T>;
type Points<Q> = Box<dyn Iterator<Item = Coordinate<Q>>>;

pub struct Path<Q: Quantity> {
    pub curve: Points<Q>,
    pub color: LayerData,
    pub width: Option<Q>,
}

impl<Q> Debug for Path<Q>
where
    Q: Quantity,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Path {{ curve: ..., (layer, datatype): {}, width: {:?} }}",
            self.color, self.width
        )
    }
}

pub struct Polygon<Q: Quantity> {
    pub area: Points<Q>,
    pub color: LayerData,
}

impl<Q> Debug for Polygon<Q>
where
    Q: Quantity,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Path {{ curve: ..., (layer, datatype): {}}}", self.color)
    }
}

#[derive(Debug, Clone)]
pub struct Ref<Q>
where
    Q: Quantity,
{
    pub(crate) strans: Option<gds21::GdsStrans>,
    pub(crate) pos: Coordinate<Q>,
    pub(crate) id: String,
    pub(crate) dep: BTreeSet<Rc<DgirCell<Q>>>, //TODO need to avoid circular ref, or dead loop happens
}

impl<Q: Quantity> Ref<Q> {
    pub fn set_pos<C: Into<Coordinate<Q>>>(&mut self, c: C) -> &mut Self {
        self.pos = c.into();
        self
    }
    pub fn set_rot<T: Num + FloatConst + FromPrimitive>(&mut self, ang: Angle<T>) -> &mut Self {
        if let Some(ref mut s) = self.strans {
            s.angle = ang.to_deg().to_f64().unwrap().into();
        } else {
            self.strans = GdsStrans {
                angle: ang.to_deg().to_f64().unwrap().into(),
                ..Default::default()
            }
            .into()
        }
        self
    }
    pub fn to_array_ref(
        &self,
        start: impl Into<Coordinate<Q>>,
        rows: i16,
        row_end: impl Into<Coordinate<Q>>,
        cols: i16,
        col_end: impl Into<Coordinate<Q>>,
    ) -> ArrayRef<Q> {
        ArrayRef {
            strans: self.strans.clone(),
            rows,
            cols,
            start: start.into(),
            col_end: col_end.into(),
            row_end: row_end.into(),
            id: self.id.clone(),
            dep: self.dep.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayRef<Q>
where
    Q: Quantity,
{
    pub(crate) strans: Option<gds21::GdsStrans>,
    pub(crate) rows: i16,
    pub(crate) cols: i16,
    pub(crate) start: Coordinate<Q>,
    pub(crate) col_end: Coordinate<Q>,
    pub(crate) row_end: Coordinate<Q>,
    pub(crate) id: String,
    pub(crate) dep: BTreeSet<Rc<DgirCell<Q>>>, //TODO need to avoid circular ref, or dead loop happens
}

impl<Q: Quantity> ArrayRef<Q> {
    pub fn set_start<C: Into<Coordinate<Q>>>(&mut self, c: C) -> &mut Self {
        self.start = c.into();
        self
    }
    pub fn set_col_end<C: Into<Coordinate<Q>>>(&mut self, c: C) -> &mut Self {
        self.col_end = c.into();
        self
    }
    pub fn set_row_end<C: Into<Coordinate<Q>>>(&mut self, c: C) -> &mut Self {
        self.row_end = c.into();
        self
    }
    pub fn set_rot<T: Num + FloatConst + FromPrimitive>(&mut self, ang: Angle<T>) -> &mut Self {
        if let Some(ref mut s) = self.strans {
            s.angle = ang.to_deg().to_f64().unwrap().into();
        } else {
            self.strans = GdsStrans {
                angle: ang.to_deg().to_f64().unwrap().into(),
                ..Default::default()
            }
            .into()
        }
        self
    }
}

#[derive(Debug)]
pub enum Element<Q>
where
    Q: Quantity,
{
    Path(Path<Q>),
    Polygon(Polygon<Q>),
    Ref(Ref<Q>),
    ARef(ArrayRef<Q>),
}

impl<Q> Element<Q>
where
    Q: Quantity,
{
    pub fn into_cell<S: ToString>(self, name: S) -> DgirCell<Q> {
        DgirCell {
            name: name.to_string(),
            elements: vec![self],
        }
    }
}

pub enum ElementsGroup<Q>
where
    Q: Quantity,
{
    Single(Element<Q>),
    Group(Vec<Element<Q>>),
}

impl<Q: Quantity> Default for ElementsGroup<Q> {
    fn default() -> Self {
        Self::Group(Vec::new())
    }
}

impl<Q: Quantity> ElementsGroup<Q> {
    pub(crate) fn as_vec(&mut self) -> &mut Vec<Element<Q>> {
        let s = std::mem::take(self);
        let g = match s {
            Self::Group(g) => g,
            Self::Single(s) => vec![s],
        };
        *self = g.into();
        if let Self::Group(g) = self {
            g
        } else {
            unreachable!()
        }
    }
    pub(crate) fn into_vec(self) -> Vec<Element<Q>> {
        match self {
            Self::Single(s) => vec![s],
            Self::Group(g) => g,
        }
    }
    pub(crate) fn extend(&mut self, other: Self) -> &mut Self {
        self.as_vec().extend(other.into_vec());
        self
    }
}

impl<Q> From<Element<Q>> for ElementsGroup<Q>
where
    Q: Quantity,
{
    fn from(e: Element<Q>) -> Self {
        ElementsGroup::Single(e)
    }
}

impl<Q> From<Vec<Element<Q>>> for ElementsGroup<Q>
where
    Q: Quantity,
{
    fn from(e: Vec<Element<Q>>) -> Self {
        ElementsGroup::Group(e)
    }
}

impl<Q> From<Path<Q>> for Element<Q>
where
    Q: Quantity,
{
    fn from(p: Path<Q>) -> Self {
        Element::Path(p)
    }
}

impl<Q> From<Polygon<Q>> for Element<Q>
where
    Q: Quantity,
{
    fn from(p: Polygon<Q>) -> Self {
        Element::Polygon(p)
    }
}

impl<Q> From<Ref<Q>> for Element<Q>
where
    Q: Quantity,
{
    fn from(r: Ref<Q>) -> Self {
        Element::Ref(r)
    }
}
impl<Q> From<ArrayRef<Q>> for Element<Q>
where
    Q: Quantity,
{
    fn from(r: ArrayRef<Q>) -> Self {
        Element::ARef(r)
    }
}

impl<Q> From<Path<Q>> for ElementsGroup<Q>
where
    Q: Quantity,
{
    fn from(p: Path<Q>) -> Self {
        Element::Path(p).into()
    }
}

impl<Q> From<Polygon<Q>> for ElementsGroup<Q>
where
    Q: Quantity,
{
    fn from(p: Polygon<Q>) -> Self {
        Element::Polygon(p).into()
    }
}

impl<Q> From<Ref<Q>> for ElementsGroup<Q>
where
    Q: Quantity,
{
    fn from(r: Ref<Q>) -> Self {
        Element::Ref(r).into()
    }
}

impl<Q> From<ArrayRef<Q>> for ElementsGroup<Q>
where
    Q: Quantity,
{
    fn from(r: ArrayRef<Q>) -> Self {
        Element::ARef(r).into()
    }
}
#[derive(Debug)]
pub struct DgirCell<Q = AbsoluteLength<f64>>
where
    Q: Quantity,
{
    pub name: String,
    pub(crate) elements: Vec<Element<Q>>,
}

impl<Q: Quantity> AsMut<DgirCell<Q>> for DgirCell<Q> {
    fn as_mut(&mut self) -> &mut DgirCell<Q> {
        self
    }
}

impl<Q> DgirCell<Q>
where
    Q: Quantity,
{
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            elements: Vec::new(),
        }
    }
    pub fn rename(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn push<U: Into<ElementsGroup<Q>>>(&mut self, element: U) -> &mut Self {
        match element.into() {
            ElementsGroup::Single(s) => self.elements.push(s),
            ElementsGroup::Group(g) => self.elements.extend(g),
        }
        self
    }

    pub fn into_ref(self) -> Ref<Q> {
        let mut s = self;
        let name = s.name.clone();
        let mut dep = s.get_dependencies();
        dep.insert(Rc::new(s));
        Ref {
            strans: None,
            dep,
            pos: Coordinate::from([Q::zero(), Q::zero()]),
            id: name,
        }
    }
    pub fn into_ref_at(self, pos: impl Into<Coordinate<Q>>) -> Ref<Q> {
        let mut s = self;
        let name = s.name.clone();
        let mut dep = s.get_dependencies();
        dep.insert(Rc::new(s));
        Ref {
            strans: None,
            dep,
            pos: pos.into(),
            id: name,
        }
    }
    pub fn into_array_ref(
        self,
        start: impl Into<Coordinate<Q>>,
        rows: i16,
        row_end: impl Into<Coordinate<Q>>,
        cols: i16,
        col_end: impl Into<Coordinate<Q>>,
    ) -> ArrayRef<Q> {
        let mut s = self;
        let mut dep = s.get_dependencies();
        let name = s.name.clone();
        dep.insert(Rc::new(s));
        ArrayRef {
            rows,
            cols,
            start: start.into(),
            col_end: col_end.into(),
            row_end: row_end.into(),
            id: name,
            dep,
            strans: None,
        }
    }
    //make sure every sub dependencies is empty
    pub(crate) fn get_dependencies(&mut self) -> BTreeSet<Rc<DgirCell<Q>>> {
        let mut dependencies = BTreeSet::new();
        for element in self.elements.iter_mut() {
            match element {
                Element::Ref(Ref { dep: ref mut d, .. }) => {
                    debug_assert!(is_sub_dependencies_empty(d));
                    dependencies.append(d);
                }
                Element::ARef(ArrayRef { dep: ref mut d, .. }) => {
                    debug_assert!(is_sub_dependencies_empty(d));
                    dependencies.append(d);
                }
                _ => (),
            }
        }
        dependencies
    }
}

impl<T: Num + FromPrimitive + ToPrimitive> DgirCell<Length<Absolute, T>> {
    pub fn save_as_lib(self, filename: impl AsRef<std::path::Path>) -> Result<()> {
        DgirLibrary {
            name: None,
            units: DgirUnits::default(),
            cells: vec![self],
        }
        .save(filename)
    }
}

impl<T: Num + FromPrimitive + ToPrimitive> DgirCell<Length<Relative, T>> {
    pub fn save_as_lib(self, filename: impl AsRef<std::path::Path>) -> Result<()> {
        DgirLibrary {
            name: None,
            units: DgirUnits::default(),
            cells: vec![self],
        }
        .save(filename)
    }
}

fn is_sub_dependencies_empty<Q: Quantity>(set: &BTreeSet<Rc<DgirCell<Q>>>) -> bool {
    set.iter().all(|c| {
        c.elements.iter().all(|e| match e {
            Element::Ref(Ref {
                dep: dependencies, ..
            }) => dependencies.is_empty(),
            _ => true,
        })
    })
}

impl<Q> PartialEq for DgirCell<Q>
where
    Q: Quantity,
{
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
impl<Q> Eq for DgirCell<Q> where Q: Quantity {}

impl<Q> PartialOrd for DgirCell<Q>
where
    Q: Quantity,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
impl<Q> Ord for DgirCell<Q>
where
    Q: Quantity,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<Q> AsRef<Vec<Element<Q>>> for DgirCell<Q>
where
    Q: Quantity,
{
    fn as_ref(&self) -> &Vec<Element<Q>> {
        &self.elements
    }
}

impl<Q> AsMut<Vec<Element<Q>>> for DgirCell<Q>
where
    Q: Quantity,
{
    fn as_mut(&mut self) -> &mut Vec<Element<Q>> {
        &mut self.elements
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DgirUnits<T>
where
    T: Num,
{
    database: Length<Absolute, T>,
    user: Length<Absolute, T>,
}

impl<T> Default for DgirUnits<T>
where
    T: Num + FromPrimitive,
{
    fn default() -> Self {
        Self {
            database: Length::new_absolute::<crate::units::Nanometer>(T::one()),
            user: Length::new_absolute::<crate::units::Micrometer>(T::one()),
        }
    }
}

#[derive(Debug)]
pub struct DgirLibrary<T, Q>
where
    T: Num + FromPrimitive,
    Q: Quantity,
{
    pub name: Option<String>,
    pub(crate) units: DgirUnits<T>,
    pub(crate) cells: Vec<DgirCell<Q>>,
}

impl<L, T> Default for DgirLibrary<T, Length<L, T>>
where
    L: LengthType,
    T: Num + FromPrimitive,
{
    fn default() -> Self {
        Self {
            name: None,
            units: DgirUnits::default(),
            cells: Vec::new(),
        }
    }
}

impl<L, T> DgirLibrary<T, Length<L, T>>
where
    L: LengthType,
    T: Num + FromPrimitive,
{
    pub fn new<S: ToString>(name: S) -> Self {
        Self {
            name: Some(name.to_string()),
            ..Default::default()
        }
    }
    pub fn set_database_unit(&mut self, db_len: Length<Absolute, T>) -> &mut Self {
        self.units.database = db_len;
        self
    }
    pub fn set_user_unit(&mut self, user_len: Length<Absolute, T>) -> &mut Self {
        self.units.user = user_len;
        self
    }
    pub fn push<C: Into<DgirCell<Length<L, T>>>>(&mut self, cell: C) -> &mut Self {
        self.cells.push(cell.into());
        self
    }
}

impl<T> DgirLibrary<T, Length<Absolute, T>>
where
    T: Num + FromPrimitive + ToPrimitive,
{
    pub fn save(self, filename: impl AsRef<std::path::Path>) -> Result<()> {
        self.to_gds21_library().save(filename)
    }
}

impl<T> DgirLibrary<T, Length<Relative, T>>
where
    T: Num + FromPrimitive + ToPrimitive,
{
    pub fn save(self, filename: impl AsRef<std::path::Path>) -> Result<()> {
        self.to_gds21_library().save(filename)
    }
}
