---
title: Runtime42 Plugin Option Enum Hash Validation
category: zircon_runtime
report_id: Runtime42-plugin-option-enum-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime42-three-hash-validation-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime42 Plugin Option Enum Hash Validation

## Scope

This slice removes logarithmic ordered-set insertion from plugin option enum-value validation. The
validator still walks values in manifest order and reports the first duplicate with the same
option-qualified error. The membership set is private and never serialized or published.

## Change

- Replace the validation-local `BTreeSet<&String>` with `HashSet<&String>`.
- Continue borrowing enum values from the plugin option manifest without allocating validation keys.
- Preserve token validation, default membership, namespace validation, and first-error ordering.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique enum values | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Validation-key allocations | 0 | 0 |
| First duplicate error | input ordered | input ordered |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME42_PLUGIN_OPTION_ENUM_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95
to be at most 60% of ordered validation P95. Exact Windows timings remain pending the coordinator
run. P95 uses nearest-rank selection; with 17 samples the gate consumes the worst sample, and a
dedicated regression locks that boundary.

## Acceptance

- `runtime42_hash_batch_plugin_option_preserves_first_duplicate_error`
  exercises the product validator with a repeated enum value.
- `runtime42_hash_batch_plugin_option_uses_borrowed_hash_set`
  requires the borrowed hash boundary and rejects ordered membership.
- `runtime42_hash_batch_plugin_option_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- `runtime42_hash_batch_plugin_option_p95_uses_nearest_rank` requires the 17-sample P95 to select
  the final ranked sample.
- This task is queued in one Runtime42 three-task asynchronous validation batch. The batch runs
  three source contracts, 12 `runtime42_hash_batch_` Rust tests, and three exact performance rows;
  no local Cargo lane was launched.

## Remaining Parent-plan Work

Runtime42 still needs one owner-scoped compiled extension generation, atomic cross-registry
publication, revoke/replace receipts, capability closure, and product-scale plugin qualification.
This slice only improves option-local duplicate validation.
