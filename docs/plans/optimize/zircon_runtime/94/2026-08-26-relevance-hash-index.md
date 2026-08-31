---
title: Runtime94 Relevance Hash Index
category: zircon_runtime
report_id: Runtime94-relevance-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime94 Relevance Hash Index

## Scope

This slice replaces the frame-local primitive relevance `BTreeMap` with `HashMap`. Frame visibility
uses the index only for stable-instance-key point lookup while walking `bvh_instances`; entity,
stable key, bounds, render-layer, relevance, and visible-index output order therefore remain owned
by the existing aligned BVH vectors.

Duplicate stable keys retain the previous latest-entry-wins behavior. Missing keys still produce
`PrimitiveRelevance::default`, and no hash iteration reaches a frame-visible output.

## Performance Workload

The release workload builds a 16,384-entry relevance index and performs 4,096 stable key lookups
per iteration.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered relevance insertions | 16,384 | 0 |
| Ordered relevance lookups | 4,096 | 0 |
| Hash relevance entries | 0 | 16,384 |
| Hash relevance lookups | 0 | 4,096 |
| BVH order projections | unchanged | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME94_RELEVANCE_HASH_INDEX_BENCH_V1`. Acceptance requires hash build-and-lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `runtime94_relevance_hash_index_preserves_latest_key_value` covers existing,
  missing, and duplicate stable keys.
- `runtime94_relevance_hash_index_keeps_bvh_output_order` locks the private
  hash owner and the unchanged aligned BVH vector projections.
- `runtime94_relevance_hash_index_p95` reports paired release P50/P95 samples
  and enforces the 30% P95 reduction gate.
- These checks share one coordinator-managed two-task Runtime94 batch with hierarchy-index
  validation. Previous-transform and HZB remain with their required copy-complete migration unions.

## Remaining Parent-plan Work

Runtime94 still owns visibility broad-phase scalability, occlusion latency, GPU-scene residency,
indirect submission, instance lifecycle, and product-scale qualification. This slice only
converges the frame-local stable-key relevance lookup.
