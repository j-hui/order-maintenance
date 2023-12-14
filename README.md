# Order Maintenance

[![Continuous integration](https://github.com/j-hui/order-maintenance/actions/workflows/ci.yaml/badge.svg)](https://github.com/j-hui/order-maintenance/actions/workflows/ci.yaml)

**This crate is still under development and may contain nasty bugs! No stability
is guaranteed as we still are focused on testing and benchmarking.**

Totally-ordered priorities for the order maintenance problem.

Will support `no_std` environments out of the box, though it will require
`alloc` to be available.

Available implementations:

-   Bender et al. (2002)'s [tag-range relabeling](src/tag_range.rs) algorithm
-   Dietz & Sleator (1987)'s [list-range relabeling](src/list_range.rs) algorithm
-   Naive rational number priorities with [`usize` numerators](src/naive.rs) (limited insertion depth, prone to panicking)
-   Naive rational number priorities with [`BigUint` numerators](src/big.rs) (extremely inefficient for non-fork-join patterns)
