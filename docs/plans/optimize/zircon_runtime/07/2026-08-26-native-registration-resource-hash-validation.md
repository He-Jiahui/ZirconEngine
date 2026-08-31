---
title: Runtime07 Native Registration Resource Hash Validation
category: zircon_runtime
report_id: Runtime07-native-registration-resource-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Native Registration Resource Hash Validation

## Scope

This slice removes ordered-tree insertion from native registration manifest resource ID
validation. Resources continue to be validated in manifest order and the first duplicate produces
the same typed error. The borrowed membership set is private and never published.

## Change

- Replace `BTreeSet<&str>` with `HashSet<&str>` for resource ID uniqueness.
- Preserve resource field validation before duplicate admission.
- Preserve system access validation, schema validation, and first-error ordering.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique resource IDs | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Validation-key allocations | 0 | 0 |
| First duplicate error | manifest ordered | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME07_NATIVE_RESOURCE_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95 to
be at most 60% of ordered validation P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ad_runtime07_hash_resource_validation_preserves_first_duplicate_error`
  parses a real manifest with a repeated resource ID.
- `optimization_batch_20260826ad_runtime07_native_resource_validation_uses_borrowed_hash_set`
  requires borrowed hash admission and rejects ordered membership.
- `optimization_batch_20260826ad_runtime07_native_resource_hash_validation_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime07 still needs signed package trust, lifecycle-safe native unloading, schema-qualified
bridge generation, capability closure, bounded discovery, and product-scale plugin qualification.
This slice only improves native resource ID validation.
