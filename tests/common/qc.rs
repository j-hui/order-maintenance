use order_maintenance::MaintainedOrd;
use quickcheck::{Arbitrary, Gen};
use std::fmt::Debug;
use std::rc::Rc;
use std::vec::Vec;

const MAX_DECISIONS: usize = 10000;

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
    fn generate_priorities<Priority: MaintainedOrd>(&self) -> Vec<Priority> {
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
}

impl Arbitrary for Decisions {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut ds = vec![];
        let mut size: usize = 1;
        let n: usize = usize::arbitrary(g) % MAX_DECISIONS;
        // let n: usize = g.size(); // TODO: use quickcheck size rather than our own
        for _ in 0..n {
            if size > 1 && bool::arbitrary(g) {
                ds.push(Decision::Drop(usize::arbitrary(g) % size));
                size -= 1;
            } else {
                ds.push(Decision::Insert(usize::arbitrary(g) % size));
                size += 1;
            }
        }
        Decisions {
            len: ds.len(),
            decisions: Rc::new(ds),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let mut lens = Vec::new();

        // Bisect decision history
        let mut len = self.len / 2;
        while 0 < len && len < self.len - 1 {
            lens.push(Decisions {
                len,
                decisions: self.decisions.clone(),
            });
            len += (self.len - len) / 2;
        }

        if self.len > 1 {
            lens.push(Decisions {
                len: self.len - 1,
                decisions: self.decisions.clone(),
            })
        }

        Box::new(lens.into_iter())
    }
}

pub fn run_and_check<Priority: MaintainedOrd>(ds: Decisions) -> bool {
    let ps: Vec<Priority> = ds.generate_priorities();
    let mut success = true;
    if !ps.is_empty() {
        // check contiguous pairs only
        // TODO: write a separate property that checks for transitivity too? But might not be
        // necessary since the underlying labels are already transitive
        for i in 0..ps.len() - 1 {
            if ps[i] >= ps[i + 1] {
                println!("Error: ps[{}] >= ps[{}]", i, i + 1);
                success = false;
            }
        }
    }
    if !success {
        // Makes divisions clearer
        println!("Among set of {} priorities\n------", ps.len());
    }
    success
}
