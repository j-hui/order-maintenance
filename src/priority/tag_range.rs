use std::cmp::Ordering;

use super::internal::{Arena, Label, PriorityRef};

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
/// # use order_maintenance::priority::tag_range*;
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut arena = Arena::new();

        // For tag-range, the base is a special priority, so we need to use another one.
        let this = arena.insert_after(Arena::BASE + 1, arena.base());
        Self(PriorityRef::new(arena, this))
    }

    pub fn insert(&self) -> Self {
        Self(self.0.insert(|arena| {
            let this = self.0.this().as_ref(arena);

            // Before we insert anything, we first need to relabel successive priorities in
            // order to ensure labels are evenly distributed.

            // Search for how many nodes we need to relabel, and its weight
            let (count, weight) = {
                let mut count = 1;
                let mut prio = this.next().as_ref(arena);

                let mut weight = prio.label().wrapping_sub(this.label());
                while weight != 0 && weight <= count * count {
                    prio = prio.next().as_ref(arena);
                    count += 1;
                    weight = prio.label().wrapping_sub(this.label());
                }
                (count, weight)
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
            this.label()
                .wrapping_add(this.next().as_ref(arena).label().wrapping_sub(this.label()) / 2)
        }))
    }

    fn relative(&self) -> Label {
        self.0.label().wrapping_sub(self.0.base_label())
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

#[cfg(test)]
mod tests {
    use super::*;

    const SOME: usize = 500;
    const MANY: usize = 2000;

    #[test]
    fn drop_single() {
        let _p = Priority::new();
    }

    #[test]
    fn compare_two() {
        let p1 = Priority::new();
        let p2 = p1.insert();
        println!("{:#?} and {:#?}", p1.relative(), p2.relative());
        assert!(p1 < p2);
    }

    #[test]
    fn insertion() {
        let p1 = Priority::new();
        let p3 = p1.insert();
        let p2 = p1.insert();

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    #[test]
    fn transitive() {
        let p1 = Priority::new();
        let p2 = p1.insert();
        let p3 = p2.insert();

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    fn do_insert(n: usize, mut next_index: impl FnMut(usize) -> usize) {
        let mut ps = vec![Priority::new()];

        for i in 0..n {
            let i = next_index(i);
            ps.insert(i + 1, ps[i].insert())
        }

        // Compare all priorities to each other
        for i in 0..ps.len() {
            for j in i + 1..ps.len() {
                assert!(ps[i] < ps[j], "ps[{}] < ps[{}]", i, j);
            }
        }
    }

    fn do_insert_begin(n: usize) {
        let p0 = Priority::new();
        let mut ps = vec![p0.clone()];
        for _ in 0..n {
            let p = p0.insert();
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
    fn insert_some_begin() {
        do_insert(SOME, |_| 0);
        do_insert_begin(SOME);
    }

    /// BUG: this fails
    #[test]
    fn insert_some_end() {
        do_insert(SOME, |n| n);
    }

    #[test]
    fn insert_some_flipflop() {
        do_insert(SOME, |n| if n % 2 == 0 { 0 } else { n })
    }

    #[test]
    fn insert_many_begin() {
        do_insert_begin(MANY);
    }

    /// BUG: this fails
    #[test]
    fn insert_many_end() {
        do_insert(MANY, |n| n);
    }

    #[test]
    fn insert_some_begin_many_end() {
        do_insert(MANY, |n| if n < SOME { 0 } else { n })
    }

    #[test]
    fn insert_many_random() {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(42);
        do_insert(MANY, |n| rng.gen_range(0..n.max(1)));
    }
}
