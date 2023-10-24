//! Totally-ordered priorities.
//!
//! See documentation for [`Priority`].
use slotmap::{self, HopSlotMap};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

mod capas;
use capas::CAPAS;

slotmap::new_key_type! {
    /// Reference to a [`Priority`], used as a key to [`Arena::priorities`].
    struct PriorityRef;
}

impl PriorityRef {
    /// "Dereferences" this index in an arena.
    ///
    /// Basically flips the arguments of [`Arena::get()`], but since this is in postfix, it's
    /// useful for chaining a series of operations.
    fn as_ref(self, arena: &Arena) -> &PriorityInner {
        arena.get(self)
    }
}

/// Shared state between all priorities that can be compared.
#[derive(Debug, Default)]
struct Arena {
    /// Total number of priorities allocated in this arena.
    total: usize,

    /// Internal store of priorities, indexed by [`PriorityRef`].
    priorities: HopSlotMap<PriorityRef, PriorityInner>,

    /// Key to the base priority, which should never be deleted (unless the arena is dropped).
    base: PriorityRef,
}

impl Arena {
    /// Label for the initial priority allocated in this arena.
    const BASE: usize = 0;

    /// The total number of labels that can be allocated in this arena.
    // const SIZE: usize = 1 << (usize::BITS - 1);

    fn new_with_priority() -> (Self, PriorityRef) {
        let mut priorities = HopSlotMap::with_key();

        let base = priorities.insert_with_key(|k| PriorityInner {
            next: RefCell::new(k),
            prev: RefCell::new(k),
            label: RefCell::new(Arena::BASE),
            ref_count: RefCell::new(1),
        });

        let first = priorities.insert(PriorityInner {
            next: RefCell::new(base),
            prev: RefCell::new(base),
            label: RefCell::new(usize::MAX / 2),
            ref_count: RefCell::new(1),
        });

        let base_prio = priorities
            .get(base)
            .expect("base should have just been inserted");
        base_prio.set_next(first);
        base_prio.set_prev(first);

        (
            Self {
                total: 1,
                priorities,
                base,
            },
            first,
        )
    }

    /// Insert a new priority into priorities store, constructing that priority using the given
    /// closure that takes the new key as argument.
    fn insert(&mut self, f: impl FnOnce(PriorityRef) -> PriorityInner) -> PriorityRef {
        self.total += 1;
        self.priorities.insert_with_key(f)
    }

    /// Remove a priority from the priorities store.
    fn remove(&mut self, key: PriorityRef) {
        self.priorities.remove(key);
        self.total -= 1;
    }

    /// Retrieve a reference to a priority from the priorities store using a key.
    fn get(&self, key: PriorityRef) -> &PriorityInner {
        self.priorities.get(key).unwrap()
    }
}

/// Contains the actual data of a priority.
///
/// To circumvent Rust mutability rules, all fields stored in here are guarded by [`RefCell`]s.
/// Helpers are used to eliminate boilerplate, and to create a level of abstraction, beneath with
/// optimizations can take place.
#[derive(Debug)]
struct PriorityInner {
    /// Pointer to the next priority in the linked list.
    next: RefCell<PriorityRef>,
    /// Pointer to the previous priority in the linked list.
    prev: RefCell<PriorityRef>,
    /// Label that is used to numerically compare
    label: RefCell<usize>,
    /// Reference count; when this reaches zero, it will be deallocated from the [`Arena`].
    ref_count: RefCell<usize>,
}

impl PriorityInner {
    fn next(&self) -> PriorityRef {
        *self.next.borrow()
    }

    fn set_next(&self, next: PriorityRef) {
        *self.next.borrow_mut() = next;
    }

    fn prev(&self) -> PriorityRef {
        *self.prev.borrow()
    }

    fn set_prev(&self, prev: PriorityRef) {
        *self.prev.borrow_mut() = prev;
    }

    fn label(&self) -> usize {
        *self.label.borrow()
    }

    fn set_label(&self, label: usize) {
        *self.label.borrow_mut() = label;
    }

    /// Compute the "weight" of some `other` priority, relative to this.
    ///
    /// The math used for this computation is not entirely intuitive, but is discussed in detail in
    /// Dietz & Sleator and Bender et al.'s papers on the order maintenance problem.
    fn weight(&self, other: &Self) -> usize {
        other.label().wrapping_sub(self.label())
    }

    /// Increment the reference count.
    fn ref_inc(&self) {
        *self.ref_count.borrow_mut() += 1;
    }

    /// Decrement the reference count; returns true when it reaches zero (time to deallocate).
    fn ref_dec(&self) -> bool {
        *self.ref_count.borrow_mut() -= 1;
        *self.ref_count.borrow() == 0
    }
}

/// A totally-ordered priority.
///
/// These priorities implement Dietz & Sleator (1987)'s solution to the order maintenance problem,
/// which require a data structure `T` that supports insertion and comparison operations such that
/// insertion constructs an element of the next greatest priority:
///
/// ```text
/// forall t: T, t < t.insert()
/// ```
///
/// but is still lower priority than all other greater priorities:
///
/// ```text
/// forall t t': T s.t. t < t', t.insert() < t'
/// ```
///
/// Amongst a collection of `n` priorities, comparison takes constant time, while insertion takes
/// amortized `log(n)` time.
///
/// ## Usage
///
/// ```rust
/// # use order_maintenance::*;
/// let p0 = Priority::new();
/// let p2 = p0.insert();
/// let p1 = p0.insert();
/// let p3 = p2.insert();
///
/// assert!(p0 < p1);
/// assert!(p0 < p2);
/// assert!(p0 < p3);
/// assert!(p1 < p2);
/// assert!(p1 < p3);
/// assert!(p2 < p3);
/// ```
///
/// ## Memory management
///
/// Under the hood, these priorities are actually references to nodes of a circular linked list,
/// allocated from an arena. Those nodes are reference-counted, which allows these priorities to be
/// cloned. The node's reference count is decremented when this priority is dropped, and if it
/// reaches zero, the node is deallocated.
///
/// Priorities from different arenas cannot be compared with one another.
///
/// ## Algorithm
///
/// This implementation uses Dietz & Sleator (1987)'s algorithm, also called tag-range relabeling
/// (as opposed to Bender et al.'s list-range relabeling algorithm).
///
/// While Dietz & Sleator also propose a data structure that supports constant-time insertion, that
/// data structure is so overwhelmingly complex that the overhead of maintaining such a data
/// structure will overwhelm any theoretical efficiency for any reasonable number of priorities.
///
/// More recently, Bender et al. proposed an alternative solution, using a list-range relabling
/// approach. That approach is likely more efficient on real hardware, since it favors bit-wise
/// operations over multiplication and division. For now, this crate uses the possibly slower
/// tag-range relabeling approach, because it was ported from a scripting language that is better
/// suited toward floating operations. It remains to be seen which implementation is better under
/// which circumstances.
///
/// ## References
///
/// -   Paul F. Dietz and Daniel D. Sleator. _Two Algorithms for Maintaining Order in a List._ 1987.
///
/// -   Michael A. Bender, Richard Cole, Erik D. Demaine, Martin Farach-Colton, and Jack Zito.
///     _Two simplified algorithms for maintaining order in a list._ 2002.
#[derive(Debug)]
pub struct Priority {
    arena: Rc<RefCell<Arena>>,
    this: PriorityRef,
}

impl Priority {
    /// Allocate a new priority in a fresh arena.
    ///
    /// Note that priorities allocated in separate arenas cannot be compared; to construct
    /// a [`Priority`] that can be compared to some existing priority, use [`Priority::insert()`].
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (arena, key) = Arena::new_with_priority();
        Self {
            arena: Rc::new(RefCell::new(arena)),
            this: key,
        }
    }

    /// Allocate the next greatest priority after the given `self`.
    pub fn insert(&self) -> Self {
        let key = self.arena_mut(|arena| {
            let this = self.this.as_ref(arena);

            // Before we insert anything, we first need to relabel successive priorities in
            // order to ensure labels are evenly distributed.

            // Search for how many nodes we need to relabel, and its weight
            let (count, weight) = {
                let mut count = 1;
                let mut prio = this.next().as_ref(arena);
                while this.weight(prio) != 0 && this.weight(prio) <= count ^ 2 {
                    prio = prio.next().as_ref(arena);
                    count += 1;
                }
                (count, this.weight(prio))
            };

            // Now, adjust labels of those nodes
            let mut prio = this.next().as_ref(arena);
            for k in 1..count {
                // if weight == 0, then it should actually encode usize::MAX + 1.
                let weight_k = if weight == 0 {
                    // Since we can't actually represent usize::MAX + 1, we just multiply it by
                    // ((usize::MAX + 1) / 2) AKA (1 << (usize::BITS / 2)), and then multiply by 2.
                    k.wrapping_mul(1 << (usize::BITS / 2)).wrapping_mul(2)
                } else {
                    k.wrapping_mul(weight)
                };
                prio.set_label((weight_k / count).wrapping_add(this.label()));

                prio = prio.next().as_ref(arena);
            }

            // Compute new priority fields
            let new_label = // New label is half-way between this and next
                this.label().wrapping_add(this.next().as_ref(arena).label().wrapping_sub(this.label()) / 2);
            let new_prev = self.this;
            let new_next = this.next();

            // Allocate new node
            let new_key = arena.insert(|_| PriorityInner {
                next: RefCell::new(new_next),
                prev: RefCell::new(new_prev),
                label: RefCell::new(new_label),
                ref_count: RefCell::new(1),
            });

            // Fix up pointers to point to newly allocated node
            let this = self.this.as_ref(arena); // appease borrow checker (:
            this.next().as_ref(arena).set_prev(new_key); // self.next.prev = new
            this.set_next(new_key); // self.next = new

            new_key
        });

        Self {
            arena: self.arena.clone(),
            this: key,
        }
    }

    pub fn insert_tag_range(&self) -> Self {
        let key = self.arena_mut(|arena| {
            let this = self.this.as_ref(arena);
            let next = this.next().as_ref(arena);

            let mut this_lab = this.label();
            let mut next_lab = if next.label() == Arena::BASE {
                usize::MAX
            } else {
                next.label()
            };

            if this_lab + 1 == next_lab {
                // Relabeling

                // find the correct list of capacities depending onnumber of priorities already inserted
                let capas_len = CAPAS.len();
                let mut t_index = capas_len;
                for (t_index_iter, _) in CAPAS.iter().enumerate().rev() {
                    if arena.total + 1 < CAPAS[t_index_iter][63] {
                        t_index = t_index_iter;
                        break;
                    }
                }
                if t_index >= capas_len {
                    panic!("Too many priorities were inserted");
                }

                let mut i = 0;
                // let mut t_i = 1.; // idea: precompute list of t_is
                let mut range_size = 1;
                let mut range_count = 1;
                let mut internal_node_tag = this_lab;

                // the subrange is [min_lab, max_lab)
                let mut min_lab = internal_node_tag;
                let mut max_lab = internal_node_tag + 1;

                let mut begin = this;
                let mut end = this.next().as_ref(arena);

                // The density threshold is 1/T^i
                // So we want to find the smallest subrange so that count/2^i <= 1/T^i
                // or count <= (2/T)^i = CAPA[t_index][i]

                while range_size < usize::MAX {
                    while begin.label() >= min_lab {
                        range_count += 1;
                        if begin.label() == Arena::BASE {
                            begin = begin.prev().as_ref(arena);
                            break;
                        }
                        begin = begin.prev().as_ref(arena);
                    }
                    // backtrack one step (this bound is inclusive)
                    begin = begin.next().as_ref(arena);
                    range_count -= 1;

                    while end.label() < max_lab && end.label() != Arena::BASE {
                        range_count += 1;
                        end = end.next().as_ref(arena);
                    }

                    if range_count < CAPAS[t_index][i] {
                        // Range found, relabel
                        let gap = range_size / range_count;
                        let mut rem = range_size % range_count; // note: the reminder is spread out
                        let mut new_label = min_lab;

                        loop {
                            begin.set_label(new_label);
                            begin = begin.next().as_ref(arena);
                            if begin.label() == end.label() {
                                break;
                            }
                            new_label += gap;
                            if rem > 0 {
                                new_label += 1;
                                rem -= 1;
                            }
                        }

                        break;
                    } else {
                        if range_size == usize::MAX {
                            panic!("Too many priorities were inserted, the root is overflowing!");
                        }
                        i += 1;
                        // t_i *= PriorityTagRange::T;
                        range_size *= 2;
                        internal_node_tag >>= 1;
                        min_lab = internal_node_tag << i;
                        max_lab = (internal_node_tag + 1) << i;
                    }
                }
            }

            this_lab = this.label();
            next_lab = if next.label() == Arena::BASE {
                usize::MAX
            } else {
                next.label()
            };

            let new_label = (this_lab & next_lab) + ((this_lab ^ next_lab) >> 1);

            let new_prev = self.this;
            let new_next = this.next();

            // Allocate new node
            let new_key = arena.insert(|_| PriorityInner {
                next: RefCell::new(new_next),
                prev: RefCell::new(new_prev),
                label: RefCell::new(new_label),
                ref_count: RefCell::new(1),
            });

            // Fix up pointers to point to newly allocated node
            let this = self.this.as_ref(arena); // appease borrow checker (:
            this.next().as_ref(arena).set_prev(new_key); // self_inner.next.prev = new
            this.set_next(new_key); // self_inner.next = new

            new_key
        });

        Self {
            arena: self.arena.clone(),
            this: key,
        }
    }

    /// Execute callback with shared reference to arena.
    ///
    /// Ugly, but useful for not exposing [`RefCell`] or [`std::cell::Ref`].
    fn arena<T>(&self, f: impl FnOnce(&Arena) -> T) -> T {
        f(&self.arena.borrow())
    }

    /// Execute callback with mutable reference to arena.
    ///
    /// Ugly, but useful for not exposing [`RefCell`] or [`std::cell::RefMut`].
    fn arena_mut<T>(&self, f: impl FnOnce(&mut Arena) -> T) -> T {
        f(&mut self.arena.borrow_mut())
    }

    fn relative(&self) -> usize {
        self.arena(|a| {
            self.this
                .as_ref(a)
                .label()
                .wrapping_sub(a.base.as_ref(a).label())
        })
    }
}

impl Clone for Priority {
    fn clone(&self) -> Self {
        // Increment ref count of the `PriorityInner`.
        self.arena(|a| self.this.as_ref(a).ref_inc());

        Self {
            arena: self.arena.clone(),
            this: self.this,
        }
    }
}

impl PartialEq for Priority {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.arena, &other.arena) && self.this == other.this
    }
}

impl Eq for Priority {}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !Rc::ptr_eq(&self.arena, &other.arena) {
            return None;
        }

        if self.this == other.this {
            return Some(Ordering::Equal);
        }

        self.relative().partial_cmp(&other.relative())
    }
}

impl Drop for Priority {
    fn drop(&mut self) {
        self.arena_mut(|a| {
            if self.this.as_ref(a).ref_dec() {
                // Ref count reached zero; remove this node from the linked list, then deallocate
                // it from the arena.

                let next = self.this.as_ref(a).next();
                let prev = self.this.as_ref(a).prev();

                // self.next.prev = self.prev
                next.as_ref(a).set_prev(prev);

                // self.prev.next = self.next
                prev.as_ref(a).set_next(next);

                a.remove(self.this)
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // const INSERT_FN: fn(&Priority) -> Priority = Priority::insert;
    const INSERT_FN: fn(&Priority) -> Priority = Priority::insert_tag_range;

    #[test]
    fn drop_single() {
        let _p = Priority::new();
    }

    #[test]
    fn compare_two() {
        let p1 = Priority::new();
        let p2 = INSERT_FN(&p1);
        assert!(p1 < p2);
    }

    #[test]
    fn insertion() {
        let p1 = Priority::new();
        let p3 = INSERT_FN(&p1);
        let p2 = INSERT_FN(&p1);

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    #[test]
    fn transitive() {
        let p1 = Priority::new();
        let p2 = INSERT_FN(&p1);
        let p3 = INSERT_FN(&p2);

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    #[test]
    fn no_leak() {
        let a = {
            let p1 = Priority::new();
            let _p2 = INSERT_FN(&p1);
            let _p3 = INSERT_FN(&p1);
            p1.arena.clone()
        };
        assert!(a.borrow().priorities.len() == 1);
    }

    #[test]
    fn can_clone() {
        let a = {
            let p1 = Priority::new();
            let p2 = INSERT_FN(&p1);
            let p3 = INSERT_FN(&p2);

            {
                let p1 = p1.clone();

                assert!(p1 < p2);
                assert!(p2 < p3);
                assert!(p1 < p3);
            }

            assert!(p1 < p2);
            assert!(p2 < p3);
            assert!(p1 < p3);
            p1.arena.clone()
        };
        assert!(a.borrow().priorities.len() == 1);
    }

    #[test]
    fn insert_100_end() {
        let mut ps = vec![Priority::new()];
        for _ in 0..99 {
            let p = INSERT_FN(ps.last().unwrap());
            ps.push(p);
        }

        // Compare all priorities to each other
        for i in 0..ps.len() {
            for j in i + 1..ps.len() {
                assert!(ps[i] < ps[j], "ps[{}] < ps[{}]", i, j);
            }
        }
    }

    #[test]
    fn insert_100_begin() {
        let p0 = Priority::new();
        let mut ps = vec![p0.clone()];
        for _ in 0..99 {
            let p = INSERT_FN(&p0);
            ps.push(p);
        }

        for j in 1..ps.len() {
            assert!(ps[0] < ps[j], "ps[{}] < ps[{}]", 0, j);
        }

        // Compare all priorities to each other
        for i in 1..ps.len() {
            for j in i + 1..ps.len() {
                assert!(ps[i] > ps[j], "ps[{}] > ps[{}]", i, j);
            }
        }
    }

    #[test]
    fn insert_200_begin_end() {
        let mut ps = vec![Priority::new()];
        for i in 0..199 {
            if i % 2 == 0 {
                let p = INSERT_FN(ps.last().unwrap());
                ps.push(p);
            } else {
                let p = INSERT_FN(ps.first().unwrap());
                ps.insert(1, p);
            }
        }

        // Compare all priorities to each other
        for i in 0..ps.len() {
            for j in i + 1..ps.len() {
                assert!(ps[i] < ps[j], "ps[{}] < ps[{}]", i, j);
            }
        }
    }

    #[test]
    fn insert_10k_end() {
        let mut ps = vec![Priority::new()];
        for _ in 0..9_999 {
            let p = INSERT_FN(ps.last().unwrap());
            ps.push(p);
        }

        // Compare consecutive priorities to each other
        for i in 0..ps.len() - 1 {
            assert!(ps[i] < ps[i + 1], "ps[{}] < ps[{}]", i, i + 1);
        }
    }

    #[test]
    fn insert_10k_begin() {
        let p0 = Priority::new();
        let mut ps = vec![p0.clone()];
        for _ in 0..9_999 {
            let p = INSERT_FN(&p0);
            ps.push(p);
        }

        for j in 1..ps.len() - 1 {
            assert!(ps[j + 1] < ps[j], "ps[{}] < ps[{}]", j + 1, j);
        }
    }
}
