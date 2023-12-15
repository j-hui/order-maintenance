use super::utils::Decisions;
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, BenchmarkId};
use order_maintenance::MaintainedOrd;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn insert_random<Priority: MaintainedOrd>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    algo: &str,
) {
    for &n in [10, 1000, 100_000].iter() {
        group.bench_with_input(BenchmarkId::new(algo, n), &n, |b, &n| {
            let mut rng = StdRng::seed_from_u64(42);
            b.iter_batched(
                || {
                    let p0 = Priority::new();
                    let mut positions = vec![];
                    for _ in 0..n {
                        positions.push(rng.gen_range(0..=positions.len()));
                    }
                    (vec![p0], positions)
                },
                |(mut ps, positions)| {
                    for i in 0..n {
                        ps.push(ps[positions[i]].insert());
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
}
pub fn comparisons<Priority: MaintainedOrd>(group: &mut BenchmarkGroup<'_, WallTime>, algo: &str) {
    group.bench_function(algo, |b| {
        let rng = StdRng::seed_from_u64(42);
        let decisions: Vec<Priority> = Decisions::new(1000, 0.6, rng).generate_priorities_ordered();
        let mut rng2 = StdRng::seed_from_u64(42);
        b.iter_batched(
            || {
                (
                    rng2.gen_range(0..decisions.len()),
                    rng2.gen_range(0..decisions.len()),
                )
            },
            |(p1, p2)| {
                let _ = decisions[p1] < decisions[p2];
            },
            criterion::BatchSize::SmallInput,
        );
    });
}
pub fn sort<Priority: MaintainedOrd>(group: &mut BenchmarkGroup<'_, WallTime>, algo: &str) {
    group.bench_function(algo, |b| {
        b.iter_batched(
            || {
                let rng = StdRng::seed_from_u64(42);
                let decisions: Vec<Priority> =
                    Decisions::new(1000, 0.6, rng).generate_priorities_ordered();
                decisions
            },
            |mut decisions| {
                decisions.sort_by(|a, b| a.partial_cmp(b).unwrap());
            },
            criterion::BatchSize::SmallInput,
        );
    });
}
