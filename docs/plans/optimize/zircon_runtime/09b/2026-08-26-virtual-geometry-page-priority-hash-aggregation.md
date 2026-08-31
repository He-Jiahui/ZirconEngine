---
title: Runtime09B Virtual Geometry Page Priority Hash Aggregation
category: zircon_runtime
report_id: Runtime09B-virtual-geometry-page-priority-hash-aggregation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Virtual Geometry Page Priority Hash Aggregation

## Scope

This slice reduces per-frame CPU work while visible virtual-geometry clusters are aggregated into
streaming page priorities. The result feeds the product `build_virtual_geometry_plan` request path.
It does not claim the parent plan's persistent GPU-scene, GPU frontier, residency, upload,
occlusion, fallback, or backend qualification work.

## Change

- Aggregate page priorities in a `HashMap<u32, PagePriority>` instead of a `BTreeMap`.
- Keep resident-page filtering and every priority field unchanged.
- Preserve the explicit final ranking by cluster count, accumulated screen-space error, LOD,
  cluster ID, and page ID.
- Isolate aggregation/update helpers so the release gate measures the changed hot operation.

## Deterministic Performance Evidence

| 262,144 visible clusters / 32,768 pages | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-tree entry probes | 262,144 | 0 | 100% removed |
| Ordered-tree nodes | 32,768 | 0 | 100% removed |
| Hash entry probes | 0 | 262,144 | average O(1) aggregation |
| Final priority ranking | full deterministic comparator | same comparator | unchanged |

The ignored release gate alternates 17 ordered-tree and hash aggregation samples. It emits
`RUNTIME09B_VIRTUAL_GEOMETRY_PAGE_PRIORITY_HASH_BENCH_V1`; acceptance requires hash P95 to be at
most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826m_runtime09b_hash_aggregation_preserves_page_ranking` covers
  resident filtering, count/error ordering, final page-ID tie-break, and zero budget.
- `optimization_batch_20260826m_runtime09b_page_priorities_use_hash_aggregation` requires the hash
  helper while preserving the explicit deterministic tie-break.
- `optimization_batch_20260826m_runtime09b_page_priority_hash_performance_evidence` emits workload
  counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime09B still requires persistent render-scene ownership, a qualified spatial/visibility
hierarchy, GPU-driven frontier and compaction, real residency/upload lifecycle, HZB truth,
fallback parity, scratch/resource budgets, and full scale/quality evidence.
