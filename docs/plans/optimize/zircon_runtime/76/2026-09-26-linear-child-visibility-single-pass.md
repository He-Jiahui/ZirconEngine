---
title: Runtime76 Linear Child Visibility Single Pass
category: zircon_runtime
report_id: Runtime76-linear-child-visibility-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime76 linear child visibility single pass

## Scope

`resolve_linear_child_main_extents` previously queried every child once to
count nodes that occupy layout, then queried every child again while building
the axis constraints. The only consumer of the count is the total inter-child
gap. The constraint loop now increments that count for each participating
child and computes the gap immediately before solving the constraints.

The change removes one tree lookup and visibility check per child. It keeps
the original ordered constraint vector, missing-node error, nonnegative gap
and available extent, scratch reuse, and solver call. Hidden children still
occupy layout; Collapsed children still contribute a zero-size constraint and
do not add a gap.

## Correctness and performance boundary

- A focused regression checks a 100-unit horizontal container with visible,
  Collapsed, and Hidden children plus a 10-unit gap. The visible and Hidden
  children each resolve to 45 units, the Collapsed child to zero, and a
  missing child still returns `UiTreeError::MissingNode`. It compares the
  constraint and resolved vectors against a test-local copy of the original
  resolver and confirms the same missing-node error.
- The child visibility/tree-lookup count falls from two per child to one per
  child on successful calls. This is a deterministic 50% reduction in that
  phase, not a claim that complete layout time halves.
- `RUNTIME76_LINEAR_CHILD_VISIBILITY_SINGLE_PASS_BENCH_V1` compares a
  test-local copy of the original two-pass resolver against the single-pass
  resolver for 256 children, including Collapsed nodes. Both constraint and
  resolved vectors are compared before timing. The benchmark uses four
  warmups, 17 alternating sample pairs, and 64 resolves per sample. The
  release P95 target is at least 10% below the exact two-pass baseline.

## Grouped validation manifest

Coalesce with the other Runtime package changes in one managed Windows lane:

1. `cargo check -p zircon_runtime --lib` for the package batch.
2. `cargo test -p zircon_runtime --lib linear_extent_gap_counts_hidden_but_not_collapsed_children`
   plus the existing `linear_arrangement_solver_reuses_constraints_and_active_indices`
   regression in the same focused Runtime test batch.
3. `cargo test --release -p zircon_runtime --lib linear_child_visibility_single_pass_release_p95 -- --ignored --nocapture`
   grouped with release evidence.
4. `rustfmt --check --edition 2021` on the two source files and
   `git diff --check` on the source and plan records.

This implementation slice has no Cargo or release measurement. Behavior and
P95 acceptance require terminal managed validation. The broader Runtime76
layout-backend and product-scale qualification gates remain open.
