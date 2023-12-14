//! Totally-ordered priorities.
pub mod big;
mod internal;
mod label;
pub mod list_range;
pub mod naive;
pub mod tag_range;

/// TODO: doc
pub trait MaintainedOrd: PartialEq + PartialOrd {
    /// TODO: doc
    fn new() -> Self;
    /// TODO: doc
    fn insert(&self) -> Self;
}
