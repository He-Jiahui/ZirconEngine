---
title: Runtime42 Plugin Event ID Hash Validation
category: zircon_runtime
report_id: Runtime42-plugin-event-id-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime42-three-hash-validation-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime42 Plugin Event ID Hash Validation

## Scope

This slice removes logarithmic ordered-set insertion from plugin event catalog ID validation. The
validator still walks manifest events in source order and reports the first duplicate with the same
catalog-qualified error. The membership set is private and never serialized or published.

## Change

- Replace the validation-local `BTreeSet<&str>` with `HashSet<&str>`.
- Continue borrowing event IDs from the catalog manifest without allocating validation keys.
- Preserve namespace, prefix, payload schema, version, and first-error validation ordering.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique event IDs | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Validation-key allocations | 0 | 0 |
| First duplicate error | input ordered | input ordered |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME42_PLUGIN_EVENT_ID_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95 to
be at most 60% of ordered validation P95. Exact Windows timings remain pending the coordinator run.
P95 uses nearest-rank selection; with 17 samples the gate consumes the worst sample, and a dedicated
regression locks that boundary.

## Acceptance

- `runtime42_hash_batch_plugin_event_preserves_first_duplicate_error`
  exercises the product validator with a repeated event ID.
- `runtime42_hash_batch_plugin_event_uses_borrowed_hash_set` requires the borrowed production hash
  boundary and rejects ordered membership.
- `runtime42_hash_batch_plugin_event_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- `runtime42_hash_batch_plugin_event_p95_uses_nearest_rank` requires the 17-sample P95 to select the
  final ranked sample.
- This task is queued in one Runtime42 three-task asynchronous validation batch. The batch runs
  three source contracts, 12 `runtime42_hash_batch_` Rust tests, and three exact performance rows;
  no local Cargo lane was launched.

## Remaining Parent-plan Work

Runtime42 still needs one owner-scoped compiled extension generation, atomic cross-registry
publication, revoke/replace receipts, capability closure, and product-scale plugin qualification.
This slice only improves catalog-local duplicate validation.
