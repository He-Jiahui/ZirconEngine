---
title: Runtime09B Visible Spatial Query Hash Dedup
category: zircon_runtime
report_id: Runtime09B-visible-spatial-query-hash-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Visible Spatial Query Hash Dedup

## Scope

This slice reduces CPU and allocation work in the renderer-visible spatial query consumed by
Editor picking. It addresses the ordered-set query cost identified by Runtime09B P1-5 without
claiming the parent plan's persistent render-scene, spatial hierarchy, scratch-memory, upload, or
backend work.

## Change

- Merge static and dynamic stable-instance candidates into a `HashSet` instead of a `BTreeSet`.
- Use the same hash path for oversized-query fallback over already-visible entries.
- Collect matching entity IDs into a `Vec`, then sort and deduplicate once at the output boundary.
- Preserve sorted, unique public results and the existing candidate, hit, and visited-node stats.

## Deterministic Performance Evidence

| 32,768 candidate keys / 16,384 entities | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-set admissions | 49,152 | 0 | 100% removed |
| Hash candidate admissions | 0 | 32,768 | average O(1) index |
| Intermediate entity values | ordered tree | 32,768 contiguous values | one final sort/dedup |
| Public entity order | sorted unique | sorted unique | unchanged |

The ignored release gate alternates 17 two-tree and hash-plus-vector samples. It emits
`RUNTIME09B_VISIBLE_SPATIAL_QUERY_HASH_DEDUP_BENCH_V1`; acceptance requires optimized P95 to be at
most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826k_runtime09b_hash_dedup_preserves_sorted_fallback_hits` forces the
  oversized-query fallback and verifies sorted unique output plus query stats.
- `optimization_batch_20260826k_runtime09b_spatial_query_uses_hash_then_vec_dedup` requires the
  hash candidate path and final vector normalization while rejecting production `BTreeSet` use.
- `optimization_batch_20260826k_runtime09b_spatial_query_hash_dedup_performance_evidence` emits
  both P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime09B still requires persistent render-scene ownership, a qualified hierarchy and update
policy, reusable query scratch storage, product upload consumption, stable dirty-range budgets,
and full static/dynamic scene scale evidence.
