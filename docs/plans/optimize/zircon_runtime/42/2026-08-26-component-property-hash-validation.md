---
title: Runtime42 Component Property Hash Validation
category: zircon_runtime
report_id: Runtime42-component-property-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime42-three-hash-validation-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime42 Component Property Hash Validation

## Scope

This slice removes logarithmic ordered-set insertion from runtime extension component-property
validation. Validation still walks descriptor properties in source order and rejects the first
duplicate with the same typed error. The membership set is never published or serialized.

## Change

- Replace the validation-local `BTreeSet<&str>` with `HashSet<&str>`.
- Continue borrowing property names from the descriptor without allocating validation keys.
- Preserve all field checks, plugin/type prefix checks, and first-error ordering.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique properties | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Validation-key allocations | 0 | 0 |
| First duplicate error | input ordered | input ordered |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME42_COMPONENT_PROPERTY_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95
to be at most 60% of ordered validation P95. Exact Windows timings remain pending the coordinator
run. P95 uses nearest-rank selection; with 17 samples the gate consumes the worst sample, and a
dedicated regression locks that boundary.

## Acceptance

- `runtime42_hash_batch_component_preserves_first_duplicate_error`
  exercises the product validator with a repeated property.
- `runtime42_hash_batch_component_uses_borrowed_hash_set`
  requires the borrowed production hash boundary and rejects ordered-set membership.
- `runtime42_hash_batch_component_property_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- `runtime42_hash_batch_component_p95_uses_nearest_rank` requires the 17-sample P95 to select the
  final ranked sample.
- This task is queued in one Runtime42 three-task asynchronous validation batch. The batch runs
  three source contracts, 12 `runtime42_hash_batch_` Rust tests, and three exact performance rows;
  no local Cargo lane was launched.

## Remaining Parent-plan Work

Runtime42 still needs one compiled owner-scoped extension registry generation shared by bootstrap,
dynamic sessions, native/VM plugins, and export. Atomic publication, revoke/replace receipts,
capability closure, and large-catalog product qualification remain open.
