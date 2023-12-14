pub use crate::MaintainedOrd;
use std::{cell::Cell, cmp::Ordering, rc::Rc};

/// A UniquePriority that can be cloned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Priority(Rc<UniquePriority>);

impl MaintainedOrd for Priority {
    fn new() -> Self {
        Self(Rc::new(UniquePriority::new()))
    }

    fn insert(&self) -> Self {
        Self(Rc::new(self.0.insert()))
    }
}

/// A UniquePriority is a rational number `label / (2 ** depth)`.
///
/// It uses interior mutability to ensure that the following works:
///
/// ```rust
///# use order_maintenance::naive::UniquePriority;
///# use order_maintenance::naive::MaintainedOrd;
/// let l = UniquePriority::new();
/// let a = l.insert();
/// let b = l.insert();
/// assert!(b < a);
/// ```
///
/// It cannot be cloned, which is why it is safe to derive `{Partial,}Eq`.
#[derive(Debug, PartialEq, Eq)]
pub struct UniquePriority {
    label: Cell<usize>,
    depth: Cell<u32>,
}

impl MaintainedOrd for UniquePriority {
    fn new() -> Self {
        Self {
            label: Cell::new(0),
            depth: Cell::new(0),
        }
    }

    fn insert(&self) -> Self {
        // This will very quickly overflow. But that's ok, that's why this is naive xD.
        self.label.set(self.label.get().checked_mul(2).unwrap());
        self.depth.set(self.depth.get() + 1);
        Self {
            label: Cell::new(self.label.get() + 1),
            depth: Cell::new(self.depth.get()),
        }
    }
}

impl PartialOrd for UniquePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.depth.get().cmp(&other.depth.get()) {
            Ordering::Equal => self.label.get().partial_cmp(&other.label.get()),
            Ordering::Less => {
                let factor = (2_usize).pow(other.depth.get() - self.depth.get());
                (self.label.get() * factor).partial_cmp(&other.label.get())
            }
            Ordering::Greater => {
                let factor = (2_usize).pow(self.depth.get() - other.depth.get());
                self.label.get().partial_cmp(&(other.label.get() * factor))
            }
        }
    }
}
