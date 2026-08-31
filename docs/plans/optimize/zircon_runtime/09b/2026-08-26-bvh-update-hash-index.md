---
title: Runtime09B BVH Update Hash Index
category: zircon_runtime
report_id: Runtime09B-bvh-update-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B BVH Update Hash Index

## Scope

This slice removes ordered-tree lookup indexes from the per-frame BVH update delta calculation.
It changes the complexity of temporary index construction and membership probes, but it does not
remove the full current/previous scene scans or establish persistent render-scene ownership.

## Change

- Build the previous and current stable-instance lookup indexes as `HashMap<u64, &Entry>`.
- Preserve inserted and updated output order from the current instance slice.
- Preserve removed output order from the previous history slice.
- Keep full-rebuild behavior and entry equality unchanged.

## Deterministic Performance Evidence

| 65,536 current and 65,536 previous entries | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-index admissions | 131,072 | 0 | 100% removed |
| Hash-index admissions | 0 | 131,072 | average O(1) admission |
| Membership/value probes | 196,608 | 196,608 | O(log scene) to average O(1) |
| Public delta ordering | current/current/previous slice order | same | unchanged |

The ignored release gate alternates 17 ordered-index and hash-index samples. It emits
`RUNTIME09B_BVH_UPDATE_HASH_INDEX_BENCH_V1`; acceptance requires hash-index P95 to be at most 60%
of ordered-index P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826o_runtime09b_hash_indexes_preserve_delta_order` covers inserted,
  updated, and removed classification plus each public vector's order.
- `optimization_batch_20260826o_runtime09b_bvh_update_uses_hash_indexes` requires exactly two hash
  indexes and rejects an ordered lookup tree in the production section.
- `optimization_batch_20260826o_runtime09b_bvh_update_hash_index_performance_evidence` emits
  workload counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime09B still scans both complete instance generations and rebuilds two full temporary maps.
Persistent render-scene slots, dirty journals, incremental spatial hierarchy updates, stable-frame
zero rebuilds, GPU-driven visibility, and complete 10k/100k/1M qualification remain open.
