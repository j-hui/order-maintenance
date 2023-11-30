//! Totally-ordered priorities.
//!
//! See documentation for [`Priority`].

mod capas;
mod internal;
mod label;
pub mod list_range;
pub mod tag_range;

pub trait MaintainedOrd: PartialEq + PartialOrd {
    fn new() -> Self;
    fn insert(&self) -> Self;
}
