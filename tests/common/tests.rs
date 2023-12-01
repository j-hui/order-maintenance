//! Tests for order maintenance implementations.
//!
//! All the tests here are helpers defined for some implementation of the `MaintainedOrd` trait.
use order_maintenance::MaintainedOrd;

const SOME: usize = 500;
const MANY: usize = 2000;

fn do_insert<Priority: MaintainedOrd>(n: usize, mut next_index: impl FnMut(usize) -> usize) {
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

fn do_insert_begin<Priority: MaintainedOrd>(n: usize) {
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

pub fn compare_two<Priority: MaintainedOrd>() {
    let p1 = Priority::new();
    let p2 = p1.insert();
    assert!(p1 < p2);
}

pub fn insertion<Priority: MaintainedOrd>() {
    let p1 = Priority::new();
    let p3 = p1.insert();
    let p2 = p1.insert();

    assert!(p1 < p2);
    assert!(p2 < p3);
    assert!(p1 < p3);
}

pub fn transitive<Priority: MaintainedOrd>() {
    let p1 = Priority::new();
    let p2 = p1.insert();
    let p3 = p2.insert();

    assert!(p1 < p2);
    assert!(p2 < p3);
    assert!(p1 < p3);
}

pub fn drop_first<Priority: MaintainedOrd>() {
    let p1 = Priority::new().insert();
    let p2 = p1.insert();

    assert!(p1 < p2);
}

pub fn drop_middle<Priority: MaintainedOrd>() {
    let p1 = Priority::new();
    let p3 = {
        let p2 = p1.insert();
        p2.insert()
    };
    let p2 = p1.insert();

    assert!(p1 < p2);
    assert!(p2 < p3);
    assert!(p1 < p3);
}

pub fn insert_some_begin<Priority: MaintainedOrd>() {
    do_insert::<Priority>(SOME, |_| 0);
    do_insert_begin::<Priority>(SOME);
}

pub fn insert_some_end<Priority: MaintainedOrd>() {
    do_insert::<Priority>(SOME, |n| n);
}

pub fn insert_some_flipflop<Priority: MaintainedOrd>() {
    do_insert::<Priority>(SOME, |n| if n % 2 == 0 { 0 } else { n })
}

pub fn insert_many_begin<Priority: MaintainedOrd>() {
    do_insert_begin::<Priority>(MANY);
}

pub fn insert_many_end<Priority: MaintainedOrd>() {
    do_insert::<Priority>(MANY, |n| n);
}

pub fn insert_some_begin_many_end<Priority: MaintainedOrd>() {
    do_insert::<Priority>(MANY, |n| if n < SOME { 0 } else { n })
}

pub fn insert_many_random<Priority: MaintainedOrd>() {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(42);
    do_insert::<Priority>(MANY, |n| rng.gen_range(0..n.max(1)));
}
