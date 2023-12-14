mod common;

use order_maintenance::big::UniquePriority;

macro_rules! delegate_tests {
    () => {};
    (fn $test_name:ident(); $($toks:tt)*) => {
        #[test]
        fn $test_name() {
            common::tests::$test_name::<UniquePriority>();
        }
        delegate_tests!{$($toks)*}
    };
}

delegate_tests! {
    fn compare_two();
    fn insertion();
    fn transitive();
    fn drop_first();
    fn drop_middle(); // Something wrong with this

    // These only work if SOME/MANY is dropped to less than 64
    fn drop_some();
    fn drop_random();
    fn insert_some_begin();
    fn insert_some_end();
    fn insert_some_flipflop();
    fn insert_many_begin();
    fn insert_many_end();
    fn insert_some_begin_many_end();
    fn insert_many_random();
}
