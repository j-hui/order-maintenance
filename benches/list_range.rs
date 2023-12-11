mod common;
use criterion::{criterion_group, criterion_main, Criterion};
use order_maintenance::list_range::Priority;

macro_rules! create_bench_functions {
    () => {};
    ($bench_name:ident($c:ident $(,$arg:expr)*); $($toks:tt)*) => {
        $c.bench_function(stringify!(list-range_$bench_name$(_$arg)*), |b| {
            common::benches::$bench_name::<Priority>(b, $($arg,)*);
        });
        create_bench_functions!{$($toks)*}
    };
}
pub fn list_range_benchmark(c: &mut Criterion) {
    create_bench_functions!(
        insert_random(c, 10);
        insert_random(c, 1000);
        insert_random(c, 100_000);
        comparisons(c);
        sort(c);
    );
}

criterion_group!(benches, list_range_benchmark);
criterion_main!(benches);
