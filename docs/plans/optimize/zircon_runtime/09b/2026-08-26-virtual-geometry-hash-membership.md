---
title: Runtime09b Virtual Geometry Hash Membership
category: zircon_runtime
report_id: Runtime09b-virtual-geometry-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09b Virtual Geometry Hash Membership

## Scope

This slice removes two quadratic membership paths from virtual-geometry visibility planning:
priority-preserving page-request deduplication and hot-resident page classification. Sorted
history, page, cluster, and draw publication remain owned by the existing ordered collections and
source vectors.

## Change

- Pair the first-seen `requested_pages` vector with a capacity-sized `HashSet<u32>` membership
  index instead of scanning the accumulated output for every request.
- Build one `HashSet<u32>` for evictable-page membership before classifying hot resident pages.
- Share the hot-resident helper between frozen-cull and normal planning branches.

## Deterministic Performance Evidence

| Representative workload | Before | After |
|---|---:|---:|
| 4,096 cascade IDs repeated twice plus 4,096 ranked IDs | 33,554,432 linear comparisons | 12,288 average `O(1)` insert probes |
| 4,096 resident pages, 512 visible, 2,560 evictable | 5,899,520 evictable-vector comparisons | 2,560 index inserts + 3,584 average `O(1)` probes |
| Request priority | first appearance across cascade then ranked | unchanged |
| Resident-page publication order | source order | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME09B_VIRTUAL_GEOMETRY_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires both optimized P95
rows to be at most 60% of their linear baselines. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826aj_prioritized_pages_preserve_first_seen_order` covers duplicate,
  cross-source, budget, and zero-budget behavior.
- `optimization_batch_20260826aj_virtual_geometry_membership_uses_hash_indexes` requires both hash
  indexes and rejects the two production vector-membership scans.
- `optimization_batch_20260826aj_virtual_geometry_hash_membership_p95` reports four P95 values and
  enforces both 60% thresholds.

## Remaining Parent-plan Work

Runtime09b still owns GPU-driven culling, hierarchy qualification, residency feedback, occlusion,
and product-scale frame evidence. This slice only converges current CPU membership hot paths.
