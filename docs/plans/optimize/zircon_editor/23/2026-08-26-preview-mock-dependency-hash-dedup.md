---
title: Editor23 Preview Mock Dependency Hash Dedup
category: zircon_editor
report_id: Editor23-preview-mock-dependency-hash-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Preview Mock Dependency Hash Dedup

## Scope

This slice removes ordered-tree membership from UI asset preview expression dependency collection.
Dependencies continue to be published through the existing `Vec` in expression traversal order,
and the first resolved value for each `(node_id, target_path)` remains authoritative. The private
membership set is never exposed or iterated.

## Change

- Replace `BTreeSet<(String, String)>` with `HashSet<(String, String)>` for dependency deduplication.
- Preserve first-occurrence insertion into the dependency result vector.
- Preserve recursive array/table/function argument traversal and graph item formatting.
- Keep the three optimization tests in a dedicated child module so the 725-line production owner
  stays below the large-file warning threshold.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique dependency keys | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Published dependencies | first unique occurrence | unchanged |
| Published order | expression traversal order | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR23_PREVIEW_DEPENDENCY_HASH_DEDUP_BENCH_V1`. Acceptance requires hash dedup P95 to be at
most 60% of ordered dedup P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ab_editor23_hash_dependency_dedup_preserves_first_value_and_order`
  exercises the production insertion helper with an out-of-order duplicate.
- `optimization_batch_20260826ab_editor23_preview_dependency_dedup_uses_hash_membership`
  requires hash membership while retaining vector publication.
- `optimization_batch_20260826ab_editor23_preview_dependency_hash_dedup_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor23 still needs typed binding/property schemas, async bounded diagnostics and imports,
generation-qualified previews, lossless V2 editing, atomic save/reimport, and large-binding
document qualification. This slice only improves preview dependency deduplication.
