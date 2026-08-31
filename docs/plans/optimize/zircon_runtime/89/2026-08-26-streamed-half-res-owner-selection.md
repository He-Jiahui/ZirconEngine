---
title: Runtime89 Streamed Half-resolution Transparency Owner Selection
category: zircon_runtime
report_id: Runtime89-streamed-half-res-owner-selection-2026-08-26
date: 2026-08-26
session_id: root-runtime89-streamed-half-res-owner-selection-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime89 Streamed Half-resolution Transparency Owner Selection

## Scope

This slice optimizes owner selection while the render-pipeline compiler replaces the transparent
mesh pass for half-resolution transparency. It removes the temporary owner vector created on every
eligible pipeline compile. It does not change feature ordering, pass/resource remapping, attachment
operations, profile gating, particle handling, plugin replacement, or the exactly-one-owner error
contract.

## Change

- Owner discovery remains a borrowed iterator and reads only the first two matching descriptors.
- No match still returns `Ok(false)`, one match is cloned and remapped, and a second match still
  returns the existing exactly-one-owner error.
- The unique pass template is cloned only after duplicate detection, so invalid graphs do not pay
  for an unused pass clone.
- A Rust regression covers zero, unique, and duplicate owner cardinalities, while a Python source
  contract prevents reintroduction of `collect::<Vec<_>>()`.

## Deterministic Performance Evidence

The independent release model performs 32,768 owner selections over 256 descriptors with the sole
owner at the end, using 21 alternating collected/streamed sample pairs per run.

| Evidence | Collected owners | Streamed owners | Result |
|---|---:|---:|---:|
| Measured allocations | 32,768 | 0 | 100% fewer |
| Run 1 P50 | 14.415 ms | 6.217 ms | 56.87% faster |
| Run 1 P95 | 18.361 ms | 19.023 ms | 3.60% slower |
| Run 2 P50 | 6.190 ms | 2.769 ms | 55.27% faster |
| Run 2 P95 | 9.602 ms | 5.380 ms | 43.97% faster |
| Run 3 P50 | 6.340 ms | 2.642 ms | 58.34% faster |
| Run 3 P95 | 9.106 ms | 3.584 ms | 60.65% faster |

The managed gate requires exactly 32,768 legacy allocations and zero streamed allocations, at
least 30% P50 improvement, and no P95 regression greater than 10%. These are owner-selection model
results, not claims about complete frame time.

## Acceptance

- `tools.tests.test_runtime89_streamed_half_res_owner_performance_contract` passes 3/3 locally.
- Exact-file `rustfmt --check` and scoped `git diff --check` pass locally.
- The focused Rust cardinality regression, source contracts, formatting, allocation/timing model,
  and scoped diff checks are submitted together in one coordinator validation batch.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Runtime89 still needs full render-graph compilation scaling evidence, transient aliasing and
barrier-plan budgets, queue scheduling and parallel encoding validation, production graph cache
acceptance, and Editor/game execution profiling.
