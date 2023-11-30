use crate::internal::{Arena, Label, PriorityRef};
pub use crate::MaintainedOrd;
use std::cmp::Ordering;

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
/// # use order_maintenance::tag_range::*;
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Priority(PriorityRef);

impl Priority {
    fn relative(&self) -> Label {
        self.0.label() - self.0.base_label()
    }

    /// Search for how many nodes we need to relabel, and its weight
    fn check_label_range(&self, arena: &mut Arena) -> (usize, Label) {
        let this = self.0.this().as_ref(arena);
        let mut count = 1;
        let mut prio = this.next().as_ref(arena);

        let mut weight = prio.label() - this.label();
        while weight != 0 && weight <= count * count {
            prio = prio.next().as_ref(arena);
            count += 1;
            weight = prio.label() - this.label();
        }
        (count, weight)
    }

    fn redistribute_labels(&self, arena: &mut Arena, count: usize, weight: Label) {
        let this = self.0.this().as_ref(arena);

        // Now, adjust labels of those nodes
        let mut prio = this.next().as_ref(arena);
        for k in 1..count {
            // if weight == 0, then it should actually encode usize::MAX + 1.
            let weight_k = if weight == 0 {
                // Since we can't actually represent usize::MAX + 1, we just multiply it by
                // ((usize::MAX + 1) / 2) AKA (1 << (usize::BITS / 2)), and then multiply by 2.
                Label::new((k * (1 << (Label::BITS / 2))) * 2)
            } else {
                weight * k
            };
            prio.set_label(weight_k / count + this.label());

            prio = prio.next().as_ref(arena);
        }
    }

    /// Perform relabeling in the arena if necessary.
    fn relabel(&self, arena: &mut Arena) {
        // Search for how many nodes we need to relabel, and its weight
        let (count, weight) = self.check_label_range(arena);
        if count > 1 {
            self.redistribute_labels(arena, count, weight);
        }
    }

    /// Compute the next label for inserting after `self`.
    fn next_label(&self, arena: &Arena) -> Label {
        let this = self.0.this().as_ref(arena);
        // Compute new priority, which is half-way between this priority and the next
        this.label() + (this.next().as_ref(arena).label() - this.label()) / 2
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
        let mut arena = Arena::new();

        // For tag-range, the base is a special priority, so we need to use another one.
        let this = arena.insert_after(Label::MAX / 2, arena.base());
        Self(PriorityRef::new(arena, this))
    }

    fn insert(&self) -> Self {
        Self(self.0.insert(|arena| {
            self.relabel(arena);
            self.next_label(arena)
        }))
    }
}
