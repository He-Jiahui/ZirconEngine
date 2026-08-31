---
title: Editor06 Capability Snapshot Hash Union
category: zircon_editor
report_id: Editor06-capability-snapshot-hash-union-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Capability Snapshot Hash Union

## Scope

This slice reduces the cost of combining minimal-host and optional-subsystem capabilities into the
immutable editor capability snapshot. It preserves the sorted vector used by binary search and
does not change enablement persistence, plugin lifecycle authority, or capability diagnostics.

## Change

- Union owned capability names in a `HashSet<String>`.
- Materialize only unique names and sort once before publishing the snapshot.
- Preserve ascending unique capabilities and the existing binary-search lookup contract.
- Leave disabled capability and diagnostic ordering unchanged.

## Deterministic Performance Evidence

| 65,536 capability values / 8,192 unique names | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-set admissions | 65,536 | 0 | 100% removed |
| Hash-set admissions | 0 | 65,536 | average O(1) admission |
| Values sorted at publication | implicit in tree | 8,192 | unique values only |
| Snapshot order | ascending unique | ascending unique | unchanged |

The ignored release gate alternates 17 ordered-union and hash-plus-sort samples. It emits
`EDITOR06_CAPABILITY_SNAPSHOT_HASH_UNION_BENCH_V1`; acceptance requires hash-plus-sort P95 to be at
most 60% of ordered-union P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826q_editor06_hash_union_preserves_sorted_unique_capabilities` covers
  duplicate names and ascending unique publication.
- `optimization_batch_20260826q_editor06_capability_snapshot_uses_hash_union` requires hash
  admission plus output sorting and rejects an ordered production set.
- `optimization_batch_20260826q_editor06_capability_hash_union_performance_evidence` emits
  workload counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor06 still needs transactional capability persistence, generation-bound enable/disable
receipts, unified unload/hot-reload status publication, last-good recovery, and full plugin-scale
startup and interaction qualification.
