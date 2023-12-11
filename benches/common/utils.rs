use order_maintenance::MaintainedOrd;
use rand::rngs::StdRng;
use rand::Rng;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum Decision {
    Insert(usize),
    Drop(usize),
}

#[derive(Clone)]
pub struct Decisions {
    len: usize,
    decisions: Rc<Vec<Decision>>,
}

impl Debug for Decisions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decisions")
            .field("len", &self.len)
            .field("decisions", &self.decisions.as_slice()[..self.len].iter())
            .finish()
    }
}

impl Decisions {
    pub fn new(len: usize, insert_percentage: f64, mut rng: StdRng) -> Self {
        assert!((0.0..=1.0).contains(&insert_percentage));
        let mut ds = vec![];
        let mut size: usize = 1;
        let n: usize = len;
        for _ in 0..n {
            if size > 1 && rng.gen_bool(1.0 - insert_percentage) {
                ds.push(Decision::Drop(rng.gen_range(0..size)));
                size -= 1;
            } else {
                ds.push(Decision::Insert(rng.gen_range(0..size)));
                size += 1;
            }
        }
        Decisions {
            len: ds.len(),
            decisions: Rc::new(ds),
        }
    }
    pub fn generate_priorities_ordered<Priority: MaintainedOrd>(&self) -> Vec<Priority> {
        let mut ps = vec![Priority::new()];
        for &d in self.decisions.as_slice()[..self.len].iter() {
            match d {
                Decision::Insert(i) => {
                    ps.insert(i + 1, ps[i].insert());
                }
                Decision::Drop(i) => {
                    ps.remove(i);
                }
            }
        }
        ps
    }
    // pub fn generate_priorities_unordered<Priority: MaintainedOrd>(&self) -> Vec<Priority> {
    //     let mut ps = vec![Priority::new()];
    //     for &d in self.decisions.as_slice()[..self.len].iter() {
    //         match d {
    //             Decision::Insert(i) => {
    //                 ps.push(ps[i].insert());
    //             }
    //             Decision::Drop(i) => {
    //                 ps.remove(i);
    //             }
    //         }
    //     }
    //     ps
    // }
}
