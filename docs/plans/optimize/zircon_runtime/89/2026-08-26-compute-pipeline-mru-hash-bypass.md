---
title: Runtime89 Compute Pipeline MRU Hash Bypass
category: zircon_runtime
report_id: Runtime89-compute-pipeline-mru-hash-bypass-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime89 Compute Pipeline MRU Hash Bypass

## Scope

This slice adds a most-recently-used bucket fast path to the generic compute pipeline cache. A
stable render-graph compute workload now compares its complete shader, entry point, and binding
schema against the most recently used bucket before constructing the three content hashes used by
the fallback index.

The 16-entry bound, complete-key collision check, failed-pipeline memoization, LRU timestamps,
scene bind-group-layout invalidation, pipeline creation, and returned WGPU handle contracts are
unchanged. Hash misses and non-MRU hits continue through the existing collision-safe bucket map.

## Deterministic Work Model

The release workload stores one pipeline with a 32 KiB shader and eight bindings, then performs
4,096 stable hits.

| Work per workload | Before | After |
|---|---:|---:|
| Shader/entry/binding content-hash calls | 12,288 | 0 |
| Complete-key comparisons | 4,096 | 4,096 |
| Pipeline creations on hits | 0 | 0 |
| Capacity or fallback-policy changes | 0 | 0 |

Stable-hit content hashing falls by 100%; exact equality remains the collision-correctness guard.
The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME89_COMPUTE_PIPELINE_MRU_HASH_BYPASS_BENCH_V1`. Acceptance requires MRU lookup P95 to be at
least 30% below the hashed lookup path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bp_compute_pipeline_mru_preserves_lru_fallback` covers MRU hits,
  non-MRU fallback, capacity, and least-recently-used eviction.
- `optimization_batch_20260826bp_compute_pipeline_mru_eliminates_stable_hashing` executes 4,096
  stable MRU hits and locks the fast path ahead of content-hash construction.
- `optimization_batch_20260826bp_compute_pipeline_mru_hash_bypass_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime89 still owns full render-graph compilation, resource lifetime, queue scheduling, barriers,
transient aliasing, and product GPU evidence. This slice only converges stable generic-compute
pipeline lookup.
