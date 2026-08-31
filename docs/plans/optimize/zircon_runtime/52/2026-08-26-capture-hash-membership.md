---
title: Runtime52 Dynamic Scene Capture Hash Membership
category: zircon_runtime
report_id: Runtime52-dynamic-scene-capture-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime52 Dynamic Scene Capture Hash Membership

## Scope

This slice optimizes two membership boundaries in dynamic-scene capture: selecting plugin-owned
component descriptors and filtering reflected field values through serializable metadata. Entity,
component, resource, descriptor, and field publication order remain unchanged.

## Change

- Replace the borrowed required-component `BTreeSet<&str>` with `HashSet<&str>`.
- Build one capacity-sized borrowed hash set containing only serializable metadata field names.
- Filter adapter-provided field values against that set in their original input order.
- Preserve the existing final descriptor sort and all reflected value ownership.

## Deterministic Performance Evidence

| Representative workload | Before | After |
|---|---:|---:|
| 65,536 type admissions / 8,192 unique types | `O(A log U)` | expected `O(A)` |
| 16,384 descriptor membership probes | `O(log U)` each | average `O(1)` each |
| 2,048 reversed fields against 2,048 metadata rows | 2,098,176 candidate visits | 2,048 index inserts + 2,048 lookups |
| Borrowed identity allocations | 0 | 0 |
| Reflected field output order | adapter input order | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME52_CAPTURE_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires both type-membership P95 and
field-filter P95 to be at most 60% of their ordered/nested baselines. Exact Windows timings remain
pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ai_runtime52_hash_field_filter_preserves_input_order` verifies that
  unknown and non-serializable fields are removed without reordering retained fields.
- `optimization_batch_20260826ai_runtime52_capture_uses_borrowed_hash_membership` requires both
  borrowed hash indexes and rejects ordered or nested-linear membership in production.
- `optimization_batch_20260826ai_runtime52_capture_hash_membership_performance_evidence` checks
  output equivalence, reports four P95 values, and enforces both 60% thresholds.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts passed before managed
  validation submission.

## Remaining Parent-plan Work

Runtime52 still needs durable multi-process archive transactions, restart-stable revisions,
bounded migration, replace-style restore, rollback, product consumers, and scale qualification.
This slice only improves current-world dynamic-scene capture.
