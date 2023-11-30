//! Internal representation and memory management of priorities.

pub(crate) use crate::label::Label;
use slab::Slab;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

/// Index to a priority in the priority arena.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub(crate) struct PriorityKey(usize);

impl From<usize> for PriorityKey {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl PriorityKey {
    /// "Dereferences" this index in an arena.
    ///
    /// Basically flips the arguments of [`Arena::get()`], but since this is in postfix, it's
    /// useful for chaining a series of operations.
    #[inline(always)]
    pub(crate) fn as_ref(self, arena: &Arena) -> &PriorityInner {
        arena.get(self)
    }

    /// Unwrap the underlying index type.
    fn key(&self) -> usize {
        self.0
    }
}

/// Shared state between all priorities that can be compared.
#[derive(Debug)]
pub(crate) struct Arena {
    /// Total number of priorities allocated in this arena.
    total: usize,

    /// Internal store of priorities, indexed by [`PriorityRef`].
    priorities: Slab<PriorityInner>,

    /// Key to the base priority, which should never be deleted (unless the arena is dropped).
    base: PriorityKey,
}

impl Arena {
    /// Label for the initial priority allocated in this arena.
    pub(crate) const BASE: Label = Label::new(0);

    /// Construct a new arena to allocate priorities in.
    ///
    /// Comes pre-allocated with a base priority, used by tag-range relabeling.
    pub(crate) fn new() -> Self {
        let mut priorities = Slab::new();
        let base_key = priorities.vacant_key().into();
        let base = priorities
            .insert(PriorityInner {
                next: RefCell::new(base_key),
                prev: RefCell::new(base_key),
                label: RefCell::new(Arena::BASE),
                ref_count: RefCell::new(1),
            })
            .into();

        debug_assert_eq!(base_key, base);

        Self {
            total: 1,
            priorities,
            base,
        }
    }

    /// Get the base priority of the arena.
    pub(crate) fn base(&self) -> PriorityKey {
        self.base
    }

    /// Retrieve a reference to a priority from the priorities store using a key.
    pub(crate) fn get(&self, key: PriorityKey) -> &PriorityInner {
        self.priorities.get(key.key()).unwrap()
    }

    /// Total number of priorities allocated in this arena.
    pub(crate) fn total(&self) -> usize {
        self.total
    }

    /// Insert a new priority into priorities store, constructing that priority using the given
    /// closure that takes the new key as argument.
    pub(crate) fn insert_after(&mut self, label: Label, prev_key: PriorityKey) -> PriorityKey {
        self.total += 1;
        let next_key = self.get(prev_key).next();
        let new_key = self
            .priorities
            .insert(PriorityInner {
                next: RefCell::new(next_key),
                prev: RefCell::new(prev_key),
                label: RefCell::new(label),
                ref_count: RefCell::new(1),
            })
            .into();
        self.get(prev_key).set_next(new_key);
        self.get(next_key).set_prev(new_key);
        new_key
    }

    /// Remove a priority from the priorities store.
    pub(crate) fn remove(&mut self, key: PriorityKey) {
        match self.total.cmp(&2) {
            Ordering::Greater => {
                let prio = self.get(key);
                let next_key = prio.next();
                let prev_key = prio.prev();
                self.get(next_key).set_prev(prev_key);
                self.get(prev_key).set_next(next_key);
            }
            Ordering::Equal => {
                let last_key = self.get(key).next();
                let last = self.get(last_key);
                last.set_next(last_key);
                last.set_prev(last_key);
            }
            Ordering::Less => (),
        }

        self.priorities.remove(key.key());
        self.total -= 1;
    }
}

/// Contains the actual data of a priority.
///
/// To circumvent Rust mutability rules, all fields stored in here are guarded by [`RefCell`]s.
/// Helpers are used to eliminate boilerplate, and to create a level of abstraction, beneath with
/// optimizations can take place.
#[derive(Debug)]
pub(crate) struct PriorityInner {
    /// Pointer to the next priority in the linked list.
    next: RefCell<PriorityKey>,

    /// Pointer to the previous priority in the linked list.
    prev: RefCell<PriorityKey>,

    /// Label that is used to numerically compare
    label: RefCell<Label>,

    /// Reference count; when this reaches zero, it will be deallocated from the [`Arena`].
    ref_count: RefCell<usize>,
}

impl PriorityInner {
    pub(crate) fn next(&self) -> PriorityKey {
        *self.next.borrow()
    }

    pub(crate) fn set_next(&self, next: PriorityKey) {
        *self.next.borrow_mut() = next;
    }

    pub(crate) fn prev(&self) -> PriorityKey {
        *self.prev.borrow()
    }

    pub(crate) fn set_prev(&self, prev: PriorityKey) {
        *self.prev.borrow_mut() = prev;
    }

    pub(crate) fn label(&self) -> Label {
        *self.label.borrow()
    }

    pub(crate) fn set_label(&self, label: Label) {
        *self.label.borrow_mut() = label;
    }

    /// Increment the reference count.
    pub(crate) fn ref_inc(&self) {
        *self.ref_count.borrow_mut() += 1;
    }

    /// Decrement the reference count; returns true when it reaches zero (time to deallocate).
    pub(crate) fn ref_dec(&self) -> bool {
        *self.ref_count.borrow_mut() -= 1;
        *self.ref_count.borrow() == 0
    }
}

/// Smart pointer to an arena and a key to a priority in that arena.
///
/// Reference-counted; `Clone` and `Drop` are implemented so that it acts like a smart pointer.
#[derive(Debug)]
pub struct PriorityRef {
    arena: Rc<RefCell<Arena>>,
    this: PriorityKey,
}

impl PriorityRef {
    /// Allocate a new priority handle.
    pub(crate) fn new(arena: Arena, this: PriorityKey) -> Self {
        Self {
            arena: Rc::new(RefCell::new(arena)),
            this,
        }
    }

    /// Get the key
    pub(crate) fn this(&self) -> PriorityKey {
        self.this
    }

    /// Insert a new priority after this one in the arena.
    ///
    /// The callback `f` is used to:
    /// (1) perform any necessary relabeling, and
    /// (2) compute the new label.
    pub(crate) fn insert(&self, f: impl FnOnce(&mut Arena) -> Label) -> Self {
        let mut arena = self.arena.borrow_mut();
        let new_label = f(&mut arena);
        let this = arena.insert_after(new_label, self.this());
        Self {
            arena: self.arena.clone(),
            this,
        }
    }

    /// Get the label of this priority.
    pub(crate) fn label(&self) -> Label {
        self.arena.borrow().get(self.this).label()
    }

    /// Get the label of the base priority.
    pub(crate) fn base_label(&self) -> Label {
        let a = self.arena.borrow();
        a.base().as_ref(&a).label()
    }

    /// Whether this priority is in the same arena as another.
    pub(crate) fn same_arena(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.arena, &other.arena)
    }
}

impl Clone for PriorityRef {
    fn clone(&self) -> Self {
        // Increment ref count of the `PriorityInner`.
        self.arena.borrow().get(self.this).ref_inc();

        Self {
            arena: self.arena.clone(),
            this: self.this,
        }
    }
}

impl Drop for PriorityRef {
    fn drop(&mut self) {
        let mut a = self.arena.borrow_mut();
        if a.get(self.this).ref_dec() {
            // Ref count reached zero; remove this node from the linked list, then deallocate
            // it from the arena.
            a.remove(self.this);
        }
    }
}

impl PartialEq for PriorityRef {
    fn eq(&self, other: &Self) -> bool {
        self.same_arena(other) && self.this == other.this
    }
}

impl Eq for PriorityRef {}

/*
TODO: recover this
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_leak() {
        // TODO
        // let a = {
        //     let a = Arena::new();
        //     let p1 = PriorityRef::new(a, 0);
        //     let _p2 = p1.insert();
        //     let _p3 = p1.insert();
        //     p1.0.arena.clone()
        // };
        // assert!(a.borrow().priorities.len() == 1);
    }

    // #[test]
    // fn can_clone() {
    //     let a = {
    //         let p1 = Priority::new();
    //         let p2 = INSERT_FN(&p1);
    //         let p3 = INSERT_FN(&p2);
    //
    //         {
    //             let p1 = p1.clone();
    //
    //             assert!(p1 < p2);
    //             assert!(p2 < p3);
    //             assert!(p1 < p3);
    //         }
    //
    //         assert!(p1 < p2);
    //         assert!(p2 < p3);
    //         assert!(p1 < p3);
    //         p1.arena.clone()
    //     };
    //     assert!(a.borrow().priorities.len() == 1);
    // }
}
*/
