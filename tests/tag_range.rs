//! Integration tests for tag-range implementation.
//!
//! Delegates to tests defined in the `common` module.

mod common;
use common::quickcheck::{qc_ordered_common, Decisions};
use order_maintenance::tag_range::Priority;
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
fn qc_ordered(ds: Decisions) -> bool {
    qc_ordered_common::<Priority>(ds)
}
