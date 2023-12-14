//! Totally-ordered priorities.
//!
//! See documentation for [`Priority`].

mod internal;
mod label;
pub mod list_range;
pub mod tag_range;
pub mod naive;
pub mod big;

/// TODO: doc
pub trait MaintainedOrd: PartialEq + PartialOrd {
    /// TODO: doc
    fn new() -> Self;
    /// TODO: doc
    fn insert(&self) -> Self;
}
