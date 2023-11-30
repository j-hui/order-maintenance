//! Totally-ordered priorities.
//!
//! See documentation for [`Priority`].

mod capas;
mod internal;
mod label;
pub mod list_range;
pub mod tag_range;

pub trait MaintainedOrd: PartialEq + PartialOrd {
    fn new() -> Self;
    fn insert(&self) -> Self;
}

/// Tests for order maintenance implementations.
///
/// All the tests here are helpers defined for some implementation of the `MaintainedOrd` trait.
#[cfg(test)]
pub(crate) mod tests {
    use crate::MaintainedOrd;

    pub(crate) const SOME: usize = 500;
    pub(crate) const MANY: usize = 2000;

    pub(crate) fn do_insert<Priority: MaintainedOrd>(
        n: usize,
        mut next_index: impl FnMut(usize) -> usize,
    ) {
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

    pub(crate) fn do_insert_begin<Priority: MaintainedOrd>(n: usize) {
        let mut ps = vec![Priority::new()];
        for _ in 0..n {
            let p = ps[0].insert();
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

    pub(crate) fn compare_two<Priority: MaintainedOrd>() {
        let p1 = Priority::new();
        let p2 = p1.insert();
        assert!(p1 < p2);
    }

    pub(crate) fn insertion<Priority: MaintainedOrd>() {
        let p1 = Priority::new();
        let p3 = p1.insert();
        let p2 = p1.insert();

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    pub(crate) fn transitive<Priority: MaintainedOrd>() {
        let p1 = Priority::new();
        let p2 = p1.insert();
        let p3 = p2.insert();

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    pub(crate) fn insert_some_begin<Priority: MaintainedOrd>() {
        do_insert::<Priority>(SOME, |_| 0);
        do_insert_begin::<Priority>(SOME);
    }

    pub(crate) fn insert_some_end<Priority: MaintainedOrd>() {
        do_insert::<Priority>(SOME, |n| n);
    }

    pub(crate) fn insert_some_flipflop<Priority: MaintainedOrd>() {
        do_insert::<Priority>(SOME, |n| if n % 2 == 0 { 0 } else { n })
    }

    pub(crate) fn insert_many_begin<Priority: MaintainedOrd>() {
        do_insert_begin::<Priority>(MANY);
    }

    pub(crate) fn insert_many_end<Priority: MaintainedOrd>() {
        do_insert::<Priority>(MANY, |n| n);
    }

    pub(crate) fn insert_some_begin_many_end<Priority: MaintainedOrd>() {
        do_insert::<Priority>(MANY, |n| if n < SOME { 0 } else { n })
    }

    pub(crate) fn insert_many_random<Priority: MaintainedOrd>() {
        use rand::{rngs::StdRng, Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(42);
        do_insert::<Priority>(MANY, |n| rng.gen_range(0..n.max(1)));
    }
}
