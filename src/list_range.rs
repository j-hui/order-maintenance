use crate::internal::{Arena, Label, PriorityRef};
pub use crate::MaintainedOrd;
use order_maintenance_macros::generate_capacities;
use std::cmp::Ordering;

generate_capacities! {
    /// Capacities for 17 thresholds in the range `(1.1..=1.9)` (inclusive) with 64-bit tags.
    const CAPACITIES: [[1.1..=1.9; 64]; 17];
}

/// A totally-ordered priority.
///
/// These priorities implement Bender et al. (2002)'s solution to the order maintenance problem,
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
/// # use order_maintenance::list_range::*;
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Priority(PriorityRef);

impl Priority {
    fn relative(&self) -> Label {
        self.0.label()
    }

    /// Find the correct list of capacities depending onnumber of priorities already inserted.
    fn threshold_index(&self, total: usize) -> usize {
        // find the correct list of capacities depending onnumber of priorities already inserted
        for (i, _) in CAPACITIES.iter().enumerate().rev() {
            let last = *unsafe { CAPACITIES[i].last().unwrap_unchecked() };
            if total + 1 < last {
                return i;
            }
        }

        panic!("Too many priorities were inserted: {total}");
    }

    /// Perform relabeling in the arena.
    fn do_relabel(&self, arena: &mut Arena) {
        // Relabeling
        let this = self.0.this().as_ref(arena);

        let t_index = self.threshold_index(arena.total());

        let mut i = 0;
        // let mut t_i = 1.; // idea: precompute list of t_is
        let mut range_size = 1;
        let mut range_count = 1;
        let mut internal_node_tag = this.label();

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
                end = end.next().as_ref(arena)
            }

            if range_count < CAPACITIES[t_index][i] {
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
                // t_i *= Priority::T;
                range_size *= 2;
                internal_node_tag >>= 1;
                min_lab = internal_node_tag << i;
                max_lab = (internal_node_tag + 1) << i;
            }
        }
    }

    /// Perform relabeling in the arena if necessary.
    fn relabel(&self, arena: &mut Arena) {
        let this = self.0.this().as_ref(arena);
        let next = this.next().as_ref(arena);
        let next_lab = if next.label() == Arena::BASE {
            Label::MAX
        } else {
            next.label()
        };

        if this.label() + 1 == next_lab {
            self.do_relabel(arena)
        }
    }

    /// Compute the next label for inserting after `self`.
    fn next_label(&self, arena: &Arena) -> Label {
        let this = self.0.this().as_ref(arena);
        let next = this.next().as_ref(arena);
        let next_lab = if next.label() == Arena::BASE {
            Label::MAX
        } else {
            next.label()
        };

        (this.label() & next_lab) + ((this.label() ^ next_lab) >> 1)
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if !self.0.same_arena(&other.0) {
            None
        } else if self.0 == other.0 {
            Some(Ordering::Equal)
        } else {
            self.relative().partial_cmp(&other.relative())
        }
    }
}

impl MaintainedOrd for Priority {
    fn new() -> Self {
        let arena = Arena::new();
        // Base is not a specially designated priority in this implementation, so we
        // can use it as the first priority.
        let this = arena.base();
        Priority(PriorityRef::new(arena, this))
    }

    fn insert(&self) -> Self {
        Self(self.0.insert(|arena| {
            self.relabel(arena);
            self.next_label(arena)
        }))
    }
}
