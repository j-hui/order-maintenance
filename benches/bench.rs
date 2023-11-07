use criterion::{criterion_group, criterion_main, Criterion};
use order_maintenance::Priority;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub fn insert_benchmark(c: &mut Criterion) {
    for insert_fn_name in ["list-range", "tag-range"] {
        let mut group = c.benchmark_group(insert_fn_name);
        let insert_fn = match insert_fn_name {
            "list-range" => Priority::insert,
            "tag-range" => Priority::insert_tag_range,
            _ => unreachable!(),
        };
        for k in [10, 100, 1_000, 10_000, 100_000, 1_000_000, 5_000_000].iter() {
            group.bench_with_input(
                &format!("insert_{insert_fn_name}_{k}_random"),
                k,
                |b, &k| {
                    b.iter(|| {
                        let mut ps = vec![Priority::new()];
                        let mut rng = StdRng::seed_from_u64(42);

                        for _ in 0..k {
                            let i = rng.gen_range(0..ps.len());
                            ps.push(insert_fn(&ps[i]));
                        }
                    })
                },
            );
            // c.bench_function(&format!("insert_{insert_fn_name}_{k}_end"), |b| {
            //     b.iter(|| {
            //         let mut ps = vec![Priority::new()];
            //         for _ in 0..k {
            //             ps.push(insert_fn(ps.last().unwrap()));
            //         }
            //     })
            // });
            // c.bench_function(&format!("insert_{insert_fn_name}_{k}_begin"), |b| {
            //     b.iter(|| {
            //         let mut ps = vec![Priority::new()];
            //         for _ in 0..k {
            //             ps.push(insert_fn(&ps[0]));
            //         }
            //     })
            // });
        }
    }
}

criterion_group!(benches, insert_benchmark);
criterion_main!(benches);
