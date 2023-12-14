pub use crate::MaintainedOrd;
use num::{bigint::BigUint, Zero};
use std::{
    cell::{Cell, UnsafeCell},
    cmp::Ordering,
    fmt::Debug,
    rc::Rc,
};

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
pub struct UniquePriority {
    label: UnsafeCell<BigUint>,
    depth: Cell<u32>,
}

impl Debug for UniquePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UniquePriority")
            .field("label", unsafe { &*self.label.get() })
            .field("depth", &self.depth)
            .finish()
    }
}

impl PartialEq for UniquePriority {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.label.get() == *other.label.get() && self.depth == other.depth }
    }
}

impl Eq for UniquePriority {}

impl MaintainedOrd for UniquePriority {
    fn new() -> Self {
        Self {
            label: UnsafeCell::new(Zero::zero()),
            depth: Cell::new(0),
        }
    }

    fn insert(&self) -> Self {
        let new_label;
        unsafe {
            *self.label.get() *= 2_u8;
            new_label = (*self.label.get()).clone() + 1_u8;
        }
        self.depth.set(self.depth.get() + 1);
        Self {
            label: UnsafeCell::new(new_label),
            depth: Cell::new(self.depth.get()),
        }
    }
}

impl PartialOrd for UniquePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.depth.get().cmp(&other.depth.get()) {
            Ordering::Equal => unsafe { (*self.label.get()).partial_cmp(&*other.label.get()) },
            Ordering::Less => {
                let factor = BigUint::new(vec![2]).pow(other.depth.get() - self.depth.get());
                let adjusted_label = unsafe { (*self.label.get()).clone() } * factor;
                unsafe { adjusted_label.partial_cmp(&*other.label.get()) }
            }
            Ordering::Greater => {
                let factor = BigUint::new(vec![2]).pow(self.depth.get() - other.depth.get());
                let adjusted_label = unsafe { (*other.label.get()).clone() } * factor;
                unsafe { (*self.label.get()).partial_cmp(&adjusted_label) }
            }
        }
    }
}
