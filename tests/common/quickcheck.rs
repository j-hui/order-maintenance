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
pub struct Decisions(pub Vec<Decision>);

impl<Priority: MaintainedOrd> From<Decisions> for Vec<Priority> {
    fn from(ds: Decisions) -> Self {
        let mut ps = vec![Priority::new()];
        for &d in ds.0.iter() {
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

// impl Debug for Decisions {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut res_str = String::new();
//         res_str.push_str("\n\n");
//         for d in self.0.iter() {
//             match d {
//                 Decision::Insert(i) => f.write_str(format!("I({}), ", i).as_str())?,
//                 Decision::Drop(i) => f.write_str(format!("D({}), ", i).as_str())?,
//             }
//         }
//         Ok(())
//         // if !self.0.is_empty() {
//         //     res_str.push_str("\n\nBefore last: ");
//         //     let vec = Vec::<Priority>::from(Decisions(self.0.clone()[..self.0.len() - 1].to_vec()));
//         //     if let Some(p0) = vec.first() {
//         //         p0.arena_mut(|a| {
//         //             for (_, p) in vec.iter().enumerate() {
//         //                 res_str.push_str(a.get(p.this).label().to_string().as_str());
//         //                 res_str.push_str(", ");
//         //             }
//         //         });
//         //     }
//         // }
//         // res_str.push_str("\n\nAfter last: ");
//         // let vec = Vec::<Priority>::from(self.clone());
//         // if let Some(p0) = vec.first() {
//         //     p0.arena_mut(|a| {
//         //         for (_, p) in vec.iter().enumerate() {
//         //             res_str.push_str(a.get(p.this).label().to_string().as_str());
//         //             res_str.push_str(", ");
//         //         }
//         //     });
//         // }
//         // res_str.push_str("\n\n");
//         // res_str
//         //     .push_str(format!("Decisions: {} - Priorities: {}", self.0.len(), vec.len()).as_str());
//         // res_str.push_str("\n\n");
//         // write!(f, "{}", res_str)
//     }
// }

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

pub fn qc_ordered_common<Priority: MaintainedOrd>(ds: Decisions) -> bool {
    let ps: Vec<Priority> = ds.clone().into();
    if !ps.is_empty() {
        // check contiguous pairs only
        for i in 0..ps.len() - 1 {
            if ps[i] >= ps[i + 1] {
                println!("ps[{}] >= ps[{}]", i, i + 1);
                return false;
            }
        }
    }
    true
}
