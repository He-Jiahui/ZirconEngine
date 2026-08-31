---
title: Runtime74 Hot Reload Hash Admission
category: zircon_runtime
report_id: Runtime74-hot-reload-hash-admission-2026-08-26
date: 2026-08-26
session_id: root-runtime74-three-task-hash-streaming-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 Hot Reload Hash Admission

## Scope

This slice removes ordered membership sets from UI asset hot-reload planning and avoids allocating
owned IDs for duplicate admissions. It preserves first-seen work ordering but does not implement
transactional tree replacement, state migration, binding reinstallation, or rollback.

## Change

- Use a borrowed `HashSet<&str>` for removed-template membership.
- Use seven `HashSet<String>` instances for builder-level first-seen admission.
- Check borrowed membership before allocating an owned ID for a duplicate.
- Keep every published work vector in report/target first-seen order.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique assets | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-set insert attempts | 65,536 | 0 | 100% removed |
| Hash membership probes | 0 | 65,536 | average O(1) |
| Owned string allocations | 73,728 | 16,384 | 77.8% removed |
| Published target order | first-seen | first-seen | unchanged |

The product builder applies the same admission helper to seven work channels. The ignored release
gate benchmarks one representative channel with 17 alternating samples and emits
`RUNTIME74_HOT_RELOAD_HASH_ADMISSION_BENCH_V1`; acceptance requires hash-admission P95 to be at
most 60% of tree-admission P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `runtime74_batch_hash_admission_preserves_first_seen_order` covers
  duplicate suppression and first-seen vector order through the product helper.
- `runtime74_batch_hot_reload_uses_hash_admission_sets` requires all eight
  hash membership boundaries and rejects production tree sets.
- `runtime74_batch_hot_reload_hash_admission_performance_evidence` emits
  workload/allocation counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

The managed `runtime74_batch_` release gate seals this work together with the resource-reference
visitor and watch-invalidation slices in one Cargo invocation: three source contracts, nine Rust
tests, and three performance rows. Dynamic Windows marker values, commit attribution, and WeCom
publication remain pending the coordinator result.

## Remaining Parent-plan Work

Runtime74 still reports rebuild targets without atomically replacing trees, migrating component
state, recompiling/rebinding typed endpoints, or rolling back partial failure. Dependency-scale,
surface-scale, allocation, and complete product hot-reload qualification remain open.
