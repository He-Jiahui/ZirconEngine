---
title: Runtime09B Particle History Hash Dedup
category: zircon_runtime
report_id: Runtime09B-particle-history-hash-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Particle History Hash Dedup

## Scope

This slice removes ordered-tree admission while rebuilding the particle-emitter portion of the
visibility history snapshot. It preserves the snapshot contract but does not eliminate per-frame
history reconstruction or establish dirty-generation ownership.

## Change

- Deduplicate extracted emitter IDs in a `HashSet<EntityId>`.
- Materialize only unique IDs and sort once at the snapshot boundary.
- Preserve ascending unique particle-emitter history for downstream delta planning.

## Deterministic Performance Evidence

| 100,000 emitter values / 50,000 unique IDs | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered dedup admissions | 100,000 | 0 | 100% removed |
| Hash dedup admissions | 0 | 100,000 | average O(1) admission |
| Values sorted at output | implicit in tree | 50,000 | unique values only |
| Snapshot order | ascending unique | ascending unique | unchanged |

The ignored release gate alternates 17 ordered-dedup and hash-plus-sort samples. It emits
`RUNTIME09B_PARTICLE_HISTORY_HASH_DEDUP_BENCH_V1`; acceptance requires hash-plus-sort P95 to be at
most 60% of ordered-dedup P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826p_runtime09b_particle_history_preserves_sorted_unique_ids` covers
  unordered duplicates and the public ascending unique result.
- `optimization_batch_20260826p_runtime09b_particle_history_uses_hash_dedup` requires hash
  admission plus output sorting and rejects an ordered production set.
- `optimization_batch_20260826p_runtime09b_particle_history_hash_dedup_performance_evidence`
  emits workload counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime09B still rebuilds particle and instance history every frame. Persistent render-scene
generations, particle dirty journals, stable-frame zero rebuilds, spatial hierarchy ownership,
GPU-driven visibility, and complete scale qualification remain open.
