use std::{collections::BTreeSet, fmt::Debug, rc::Rc};

use num::{FromPrimitive, ToPrimitive, Zero};

use crate::{
    color::LayerData,
    draw::coordinate::{Coordinate, LenCo},
    units::{Absolute, Length, LengthType, Relative},
    Num,
};

use self::togds::ToGds21Library;

mod togds;

//const DISPLAY_POINTS_NUM: usize = 20;
type Result<T> = gds21::GdsResult<T>;
type Points<L, T> = Box<dyn Iterator<Item = LenCo<L, T>>>;

pub struct Path<L: LengthType, T: Num> {
    pub curve: Points<L, T>,
    pub color: LayerData,
    pub width: Option<Length<L, T>>,
}

impl<L, T> Debug for Path<L, T>
where
    L: LengthType,
    T: Num,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Path {{ curve: ..., (layer, datatype): {}, width: {:?} }}",
            self.color, self.width
        )
    }
}

pub struct Polygon<L: LengthType, T: Num> {
    pub area: Points<L, T>,
    pub color: LayerData,
}

impl<L, T> Debug for Polygon<L, T>
where
    L: LengthType,
    T: Num,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Path {{ curve: ..., (layer, datatype): {}}}", self.color)
    }
}

#[derive(Debug)]
pub struct Ref<L, T>
where
    L: LengthType,
    T: Num,
{
    pub(crate) strans: Option<gds21::GdsStrans>,
    pub(crate) pos: Coordinate<Length<L, T>>,
    pub(crate) id: String,
    pub(crate) dep: BTreeSet<Rc<DgirCell<L, T>>>, //TODO need to avoid circular ref, or dead loop happens
}

#[derive(Debug)]
pub enum Element<L, T>
where
    L: LengthType,
    T: Num,
{
    Path(Path<L, T>),
    Polygon(Polygon<L, T>),
    Ref(Ref<L, T>),
}

impl<L, T> Element<L, T>
where
    L: LengthType,
    T: Num,
{
    pub fn to_cell<S: ToString>(self, name: S) -> DgirCell<L, T> {
        DgirCell {
            name: name.to_string(),
            elements: vec![self],
        }
    }
}

pub enum ElementsGroup<L, T>
where
    L: LengthType,
    T: Num,
{
    Single(Element<L, T>),
    Group(Vec<Element<L, T>>),
}

impl<L, T> From<Element<L, T>> for ElementsGroup<L, T>
where
    L: LengthType,
    T: Num,
{
    fn from(e: Element<L, T>) -> Self {
        ElementsGroup::Single(e)
    }
}

impl<L, T> From<Vec<Element<L, T>>> for ElementsGroup<L, T>
where
    L: LengthType,
    T: Num,
{
    fn from(e: Vec<Element<L, T>>) -> Self {
        ElementsGroup::Group(e)
    }
}

pub trait ToDgirElements<L, T>
where
    L: LengthType,
    T: Num,
{
    fn to_dgir_elements(self) -> ElementsGroup<L, T>;
}

impl<F, L, T> ToDgirElements<L, T> for F
where
    L: LengthType,
    T: Num,
    F: Into<Element<L, T>>,
{
    fn to_dgir_elements(self) -> ElementsGroup<L, T> {
        ElementsGroup::Single(self.into())
    }
}

impl<L, T> From<Path<L, T>> for Element<L, T>
where
    L: LengthType,
    T: Num,
{
    fn from(p: Path<L, T>) -> Self {
        Element::Path(p)
    }
}

impl<L, T> From<Polygon<L, T>> for Element<L, T>
where
    L: LengthType,
    T: Num,
{
    fn from(p: Polygon<L, T>) -> Self {
        Element::Polygon(p)
    }
}

impl<L, T> From<Ref<L, T>> for Element<L, T>
where
    L: LengthType,
    T: Num,
{
    fn from(r: Ref<L, T>) -> Self {
        Element::Ref(r)
    }
}

#[derive(Debug)]
pub struct DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
    pub name: String,
    pub(crate) elements: Vec<Element<L, T>>,
}

impl<L, T> DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            elements: Vec::new(),
        }
    }
    pub fn rename(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn push<U: ToDgirElements<L, T>>(&mut self, element: U) -> &mut Self {
        match element.to_dgir_elements() {
            ElementsGroup::Single(s) => self.elements.push(s),
            ElementsGroup::Group(g) => self.elements.extend(g),
        }
        self
    }

    pub fn as_ref(self) -> Ref<L, T> {
        let mut s = self;
        Ref {
            strans: None,
            dep: s.get_dependencies(),
            pos: Coordinate::from([Length::zero(), Length::zero()]),
            id: s.name,
        }
    }
    //make sure every sub dependencies is empty
    pub(crate) fn get_dependencies(&mut self) -> BTreeSet<Rc<DgirCell<L, T>>> {
        let mut dependencies = BTreeSet::new();
        for element in self.elements.iter_mut() {
            match element {
                Element::Ref(Ref { dep: ref mut d, .. }) => {
                    debug_assert!(is_sub_dependencies_empty(d));
                    dependencies.append(d);
                }
                _ => (),
            }
        }
        dependencies
    }
}

fn is_sub_dependencies_empty<L: LengthType, T: Num>(set: &BTreeSet<Rc<DgirCell<L, T>>>) -> bool {
    set.iter().all(|c| {
        c.elements.iter().all(|e| match e {
            Element::Ref(Ref {
                dep: dependencies, ..
            }) => dependencies.is_empty(),
            _ => true,
        })
    })
}

impl<L, T> PartialEq for DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
impl<L, T> Eq for DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
}

impl<L, T> PartialOrd for DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
impl<L, T> Ord for DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<L, T> AsRef<Vec<Element<L, T>>> for DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
    fn as_ref(&self) -> &Vec<Element<L, T>> {
        &self.elements
    }
}

impl<L, T> AsMut<Vec<Element<L, T>>> for DgirCell<L, T>
where
    L: LengthType,
    T: Num,
{
    fn as_mut(&mut self) -> &mut Vec<Element<L, T>> {
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
pub struct DgirLibrary<L = Absolute, T = f64>
where
    L: LengthType,
    T: Num + FromPrimitive,
{
    pub name: Option<String>,
    pub(crate) units: DgirUnits<T>,
    pub(crate) cells: Vec<DgirCell<L, T>>,
}

impl<L, T> Default for DgirLibrary<L, T>
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

impl<L, T> DgirLibrary<L, T>
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
    pub fn push<C: Into<DgirCell<L, T>>>(&mut self, cell: C) -> &mut Self {
        self.cells.push(cell.into());
        self
    }
}

impl<T> DgirLibrary<Absolute, T>
where
    T: Num + FromPrimitive + ToPrimitive,
{
    pub fn save(self, filename: impl AsRef<std::path::Path>) -> Result<()> {
        self.to_gds21_library().save(filename)
    }
}

impl<T> DgirLibrary<Relative, T>
where
    T: Num + FromPrimitive + ToPrimitive,
{
    pub fn save(self, filename: impl AsRef<std::path::Path>) -> Result<()> {
        self.to_gds21_library().save(filename)
    }
}
