mod common;
use criterion::{criterion_group, criterion_main, Criterion};
use order_maintenance::big::Priority as BigPriority;
use order_maintenance::list_range::Priority as ListRangePriority;
use order_maintenance::tag_range::Priority as TagRangePriority;

macro_rules! create_bench_function_list {
    () => {};
    ($bench_name:ident($group:expr)) => {
        common::benches::$bench_name::<ListRangePriority>($group, "list-range");
    };
}
macro_rules! create_bench_function_tag {
    () => {};
    ($bench_name:ident($group:expr)) => {
        common::benches::$bench_name::<TagRangePriority>($group, "tag-range");
    };
}
macro_rules! create_bench_function_big {
    () => {};
    ($bench_name:ident($group:expr)) => {
        common::benches::$bench_name::<BigPriority>($group, "big");
    };
}
macro_rules! create_bench_functions {
    () => {};
    ($bench_name:ident($c:ident); $($toks:tt)*) => {
        let mut group = $c.benchmark_group(stringify!($bench_name));
        create_bench_function_list!{$bench_name(&mut group)}
        create_bench_function_tag!{$bench_name(&mut group)}
        create_bench_function_big!{$bench_name(&mut group)}
        group.finish();
        create_bench_functions!{$($toks)*}
    };
}

pub fn benchmark(c: &mut Criterion) {
    create_bench_functions!(
        insert_random(c);
        comparisons(c);
        sort(c);
    );
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
