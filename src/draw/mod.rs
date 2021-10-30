use crate::units::{Absolute, Length};

pub mod coordinate;
pub mod curve;
pub mod transfer;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Resolution<T = Length<Absolute, f64>> {
    MinDistance(T),
    MinNumber(usize),
}
