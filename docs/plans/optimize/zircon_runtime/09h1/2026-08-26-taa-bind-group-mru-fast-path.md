---
title: Runtime09H1 TAA Bind Group MRU Fast Path
category: zircon_runtime
report_id: Runtime09H1-taa-bind-group-mru-fast-path-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H1 TAA Bind Group MRU Fast Path

## Scope

This slice adds a most-recently-used fast path to the TAA resolve bind-group cache. A stable frame
now checks the LRU tail directly and returns the already-cloned WGPU handle without scanning or
relocating the eight-entry deque. Non-MRU hits retain the existing linear lookup and move-to-tail
behavior.

The eight-entry bound, frame-target and history-pair invalidation, sampled texture identities,
miss-only bind-group creation, and owned `wgpu::BindGroup` result contract are unchanged.

## Deterministic Work Model

The release workload fills all eight entries and performs 4,096 stable hits against the existing
LRU tail.

| Work per stable frame | Before | After |
|---|---:|---:|
| Key comparisons | 32,768 | 4,096 |
| Deque remove-and-push relocations | 4,096 | 0 |
| Bind groups created on hits | 0 | 0 |
| Capacity or fallback-policy changes | 0 | 0 |

Deterministic comparison work falls by 87.5%, while stable-hit deque relocation falls by 100%.
The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME09H1_TAA_BIND_GROUP_MRU_FAST_PATH_BENCH_V1`. Acceptance requires MRU lookup P95 to be at
least 50% below the legacy scan-and-relocate path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bo_taa_bind_group_mru_preserves_lru_order` compares legacy and MRU
  algorithms across stable and non-MRU accesses and locks identical recency order.
- `optimization_batch_20260826bo_taa_bind_group_mru_eliminates_stable_scan` locks the direct tail
  branch while preserving the fallback scan.
- `optimization_batch_20260826bo_taa_bind_group_mru_p95` reports paired release P50/P95 samples and
  enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Runtime09H1 still owns temporal history validity, motion vectors, reactive masks, reconstruction,
dynamic resolution, debug views, and product-scale capture evidence. This slice only converges the
stable TAA bind-group cache hit path.
