# Order Maintenance

[![Continuous integration](https://github.com/j-hui/order-maintenance/actions/workflows/ci.yaml/badge.svg)](https://github.com/j-hui/order-maintenance/actions/workflows/ci.yaml)

Totally-ordered priorities for the order maintenance problem.

Current implementation uses Dietz & Sleator (1987)'s tag-range relabeling
approach.

Supports `no_std` environments out of the box, though it requires `alloc` to be
available.

## Opportunities for Optimization

This crate is still in its infancy and remains to be thoroughly tested,
benchmarked, or optimized.

Here are some premature ideas for potential optimization:

- Use Bender et al. (2002)'s list-range relabeling approach instead of
  tag-range relabeling; list-range relabling favors bitwise operations over
  multiplication and division, so it should be faster.

- Rather than setting `Arena::SIZE` to `2^63`, just let it be `2^64`, and let
  overflow happen naturally. This omits numerous modulus operations that are
  currently used. This is currently unimplemented out of caution (this crate
  was ported from an implementation with explicit modulus ops), but should be
  fine in theory.

- Experiment with using different underlying allocators, between
  `slotmap::{SlotMap, HopSlotMap,DenseSlotMap}`, `slab::Slab`, plain old `Vec`
  (shifting elements on insertion and deletion), `{,A}Rc`, or even raw
  pointers.

- Re-adjust the use of `RefCell`s used internally, or even replace them with
  `UnsafeCell`s, to reduce memory footprint and dynamic checks.
