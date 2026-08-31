---
title: Editor01 Weight Heatmap Hash Generation Cache
category: zircon_editor
report_id: Editor01-weight-heatmap-hash-generation-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Weight Heatmap Hash Generation Cache

## Scope

This slice removes the full recency-queue scan from retained weight-heatmap static-field cache
hits. Static generation and plot-grid dimensions now index a `HashMap`; each entry records a
monotonic access generation for least-recently-used eviction.

The 16-entry bound, immutable `Arc<WeightHeatmapStaticField>` ownership, source generation,
bounded-grid policy, duplicate insertion behavior, and true LRU eviction remain unchanged. Access
generation overflow rebases live entries in oldest-first order before continuing.

## Deterministic Work Model

The release workload fills all 16 entries and performs 4,096 stable hits.

| Work per stable frame | Legacy | Optimized |
|---|---:|---:|
| Recency equality comparisons | 65,536 | 0 |
| Hash lookups | 0 | 4,096 |
| Recency deque rewrites | 4,096 | 0 |
| Static fields rebuilt on hits | 0 | 0 |

Deterministic recency-scan work falls by 100%. The ignored release gate runs 17 alternating sample
pairs and emits `EDITOR01_WEIGHT_HEATMAP_HASH_GENERATION_CACHE_BENCH_V1`. Acceptance requires
HashMap generation-LRU P95 to be at least 50% below the legacy `BTreeMap` plus `VecDeque::retain`
path. Exact Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bn_weight_heatmap_hash_generation_preserves_lru` covers Arc identity,
  overflow rebase, hit promotion, capacity, and least-recent eviction.
- `optimization_batch_20260826bn_weight_heatmap_hash_generation_eliminates_recency_scan` locks the
  HashMap/generation implementation and deterministic recency model.
- `optimization_batch_20260826bn_weight_heatmap_hash_generation_p95` reports paired release P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges weight-heatmap
static-field cache hits.
