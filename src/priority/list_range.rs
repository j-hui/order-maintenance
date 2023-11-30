use std::cmp::Ordering;

use super::internal::{Arena, Label, PriorityRef};

use super::capas::CAPAS;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Priority(PriorityRef);

impl Priority {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut arena = Arena::new();

        // For tag-range, the base is a special priority, so we need to use another one.
        let this = arena.insert_after(Arena::BASE, arena.base());
        Self(PriorityRef::new(arena, this))
    }

    pub fn insert(&self) -> Self {
        Self(self.0.insert(|arena| {
            let this = self.0.this().as_ref(arena);
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
                    if arena.total() + 1 < CAPAS[t_index_iter][63] {
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
                        end = end.next().as_ref(arena)
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
                        // t_i *= Priority::T;
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

            (this_lab & next_lab) + ((this_lab ^ next_lab) >> 1)
        }))
    }

    fn relative(&self) -> Label {
        self.0.label()
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
    // const MANY: usize = 10000;

    #[test]
    fn drop_single() {
        let _p = Priority::new();
    }

    #[test]
    fn compare_two() {
        let p1 = Priority::new();
        let p2 = p1.insert();
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
