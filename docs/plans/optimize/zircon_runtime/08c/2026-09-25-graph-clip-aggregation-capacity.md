---
title: Runtime08C graph clip aggregation lower-bound capacity
category: zircon_runtime
report_id: Runtime08C-graph-clip-aggregation-capacity-2026-09-25
date: 2026-09-25
session_id: root-runtime08c-graph-capacity-20260925
parent_plan: docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_lower_bound_met
---

# Runtime08C graph clip aggregation lower-bound capacity

## Scope

The animation graph `Blend` evaluator already computes the number of input
branches before it accumulates child clips. Its aggregation vector was still
created with `Vec::new()`, so a blend with many inputs paid geometric growth
until the child results were appended. This slice reserves that known input
count as a lower bound. Nested blends may still append more than one clip per
input; the reserve therefore changes capacity growth only and does not impose a
semantic limit.

## Implementation

`zircon_runtime/src/animation/manager/graph.rs` now uses
`Vec::with_capacity(input_count)` in the `AnimationGraphNodeAsset::Blend`
branch. Clip order, weights, additive behavior, visited-node handling, and
mask target projection are unchanged. A source regression contract requires
the lower-bound reserve in the production section of the owner and rejects a
fallback `Vec::new()` there.

## Evidence

- TDD RED: the production-section contract failed before the change because
  the branch used `Vec::new()`.
- GREEN: the same contract now finds the exact lower-bound reserve; the
  production owner passes Rust 1.94.1 `rustfmt --edition 2024 --check` and
  scoped `git diff --check`.
- Deterministic target: a blend with `B` inputs starts with capacity `B`, so
  the common one-clip-per-input case performs no geometric growth allocations;
  nested fan-out remains semantically unchanged.
- Current local Runtime+Editor discovery is a grouped `4,354/4,354` pass;
  the non-ignored performance-contract subset is `2,950/2,950`. These suites
  do not execute this Rust owner, so they are recorded only as repository-level
  static evidence.
- The approved Windows `core-min,animation` target also ran the complete
  current `performance_contract_tests` module in one batch: `12` tests
  discovered, `9` non-ignored passed, `0` failed, and `3` Release tests were
  ignored by their existing gate conditions. This is local Rust evidence and
  does not replace the grouped managed lane.
- The current-source local Release ignored batch passed the companion graph
  markers in one run: Runtime08C mask-target dedup p95 `13,547→698µs` and
  Runtime624 traversal p95 `345,684→149,009µs`. Managed allocator/product
  percentile evidence remains the acceptance gate.

## Acceptance boundary

Managed Windows Cargo must compile the current Runtime source and run the
focused graph contracts. A Release lane should compare the legacy
`Vec::new()` aggregation with the reserved lower-bound path over 1/8/64/256
blend inputs and report allocation count plus p50/p95. This record does not
claim that managed compilation or a product-frame percentile has passed.
