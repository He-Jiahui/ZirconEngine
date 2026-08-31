---
title: Runtime09B Frame Batching Hash Entity Sets
category: zircon_runtime
report_id: Runtime09B-frame-batching-hash-entity-sets-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Frame Batching Hash Entity Sets

## Scope

This slice removes ordered-tree admission from the three entity membership sets rebuilt during
visibility frame batching. It addresses one O(scene log scene) source called out by Runtime09B. It
does not eliminate the full scene scan or claim persistent render-scene, incremental batching,
spatial hierarchy, GPU-driven, upload, HZB, or backend completion.

## Change

- Store renderable, static, and dynamic membership in `HashSet<EntityId>` during the frame scan.
- Keep mobility classification and duplicate suppression unchanged.
- Convert each set to a vector and sort once at the `VisibilityContext` output boundary.
- Preserve ascending, unique public entity vectors and all downstream contracts.

## Deterministic Performance Evidence

| 100,000 unique mesh entities | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-tree admissions | 200,000 | 0 | 100% removed |
| Hash admissions | 0 | 200,000 | average O(1) membership |
| Final sorted entity values | implicit in trees | 200,000 | one output-boundary sort |
| Public entity order | ascending unique | ascending unique | unchanged |

The ignored release gate alternates 17 three-tree and three hash-plus-sort samples. It emits
`RUNTIME09B_FRAME_BATCHING_HASH_ENTITY_SETS_BENCH_V1`; acceptance requires optimized P95 to be at
most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826n_runtime09b_hash_entity_sets_preserve_sorted_output` covers
  unordered duplicate input and ascending unique output.
- `optimization_batch_20260826n_runtime09b_batching_uses_hash_membership_sets` requires all three
  hash fields and all three output-boundary normalizations.
- `optimization_batch_20260826n_runtime09b_batching_hash_entity_set_performance_evidence` emits
  workload counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime09B still performs a full mesh scan and rebuilds batching/history/BVH projections each
frame. Persistent render-scene ownership, dirty generations, qualified spatial indexing,
reusable scratch, GPU-driven submission, HZB truth, upload consumption, and complete scale/quality
evidence remain open.
