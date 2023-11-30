mod internal;
mod capas;
pub mod tag_range;
pub mod list_range;

pub trait MaintainOrd: PartialEq + Eq + PartialOrd {
    /// Allocate a new priority in a fresh arena.
    ///
    /// Note that priorities allocated in separate arenas cannot be compared; to construct
    /// a [`Priority`] that can be compared to some existing priority, use [`Priority::insert()`].
    fn new() -> Self;

    /// Allocate the next greatest priority.
    fn insert(&self) -> Self;
}
