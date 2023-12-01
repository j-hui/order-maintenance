//! Integration tests for list-range implementation.
//!
//! Delegates to tests defined in the `common` module.

mod common;
use common::qc;
use order_maintenance::list_range::Priority;
use quickcheck_macros::quickcheck;

macro_rules! delegate_tests {
    () => {};
    (fn $test_name:ident(); $($toks:tt)*) => {
        #[test]
        fn $test_name() {
            common::tests::$test_name::<Priority>();
        }
        delegate_tests!{$($toks)*}
    };
}

delegate_tests! {
    fn compare_two();
    fn insertion();
    fn transitive();
    fn drop_first();
    fn drop_middle();
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

#[quickcheck]
fn qc_ordered(ds: qc::Decisions) -> bool {
    qc::run_and_check::<Priority>(ds)
}
