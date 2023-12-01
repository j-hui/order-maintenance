use order_maintenance::MaintainedOrd;
use quickcheck::{Arbitrary, Gen};
use std::fmt::Debug;
use std::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub enum Decision {
    Insert(usize),
    Drop(usize),
}

#[derive(Clone, Debug)]
pub struct Decisions(Vec<Decision>);

impl Decisions {
    fn generate_priorities<Priority: MaintainedOrd>(&self) -> Vec<Priority> {
        let mut ps = vec![Priority::new()];
        for &d in self.0.iter() {
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
        let n: usize = usize::arbitrary(g) % 10_000;
        // let n: usize = g.size();
        for _ in 0..n {
            if size > 1 && bool::arbitrary(g) {
                ds.push(Decision::Drop(usize::arbitrary(g) % size));
                size -= 1;
            } else {
                ds.push(Decision::Insert(usize::arbitrary(g) % size));
                size += 1;
            }
        }
        Decisions(ds)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        // Very inefficient for now
        let vec = self.0.clone();
        let len = vec.len();
        // println!("shrink {len}");
        Box::new(
            (1..=10)
                .map(move |pow| vec[..(len - len / (1 << pow) - 1)].to_vec())
                .chain(std::iter::once(self.0[..(len - 1)].to_vec()))
                .map(Decisions),
        )
    }
}

pub fn run_and_check<Priority: MaintainedOrd>(ds: Decisions) -> bool {
    let ps: Vec<Priority> = ds.generate_priorities();
    if !ps.is_empty() {
        // check contiguous pairs only
        for i in 0..ps.len() - 1 {
            if ps[i] >= ps[i + 1] {
                println!("Error: ps[{}] >= ps[{}]", i, i + 1);
                return false;
            }
        }
    }
    true
}
