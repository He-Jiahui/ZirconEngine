---
title: Runtime09F1 IBL Bake Pipeline Fast Hit
category: zircon_runtime
report_id: Runtime09F1-ibl-bake-pipeline-fast-hit-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09F1 IBL Bake Pipeline Fast Hit

## Scope

This slice adds a complete compute-pipeline key fast hit at the start of IBL bake pipeline ensure.
A stable PMREM or SH bake command now returns its cached WGPU pipeline after one HashMap probe,
before probing the shader-module and pipeline-layout component caches.

On a miss, shader-module and layout reuse, complete shader/output-kind key equality, pipeline
creation, cache ownership, and cloned WGPU handle return semantics are unchanged. The newly created
pipeline is inserted and returned directly instead of probing the compute-pipeline map again.

## Deterministic Work Model

The release workload fills 256 pipeline keys and performs 4,096 stable hits with a long kernel key.
The conservative baseline counts shader-module plus two compute-pipeline probes and deliberately
omits the old layout probe.

| Work per workload | Before | After |
|---|---:|---:|
| Counted HashMap probes | 12,288 | 4,096 |
| Pipeline creations on hits | 0 | 0 |
| WGPU handle clones | 4,096 | 4,096 |
| Cache-key or creation-policy changes | 0 | 0 |

Counted stable-hit probes fall by 66.7%. The ignored release gate runs 17 alternating sample pairs
and emits `RUNTIME09F1_IBL_PIPELINE_FAST_HIT_BENCH_V1`. Acceptance requires fast-hit P95 to be at
least 50% below the conservative legacy probe workload. Exact Windows P50/P95 timings remain
pending the coordinator run.

## Acceptance

- `optimization_batch_20260826br_ibl_pipeline_fast_hit_reuses_cached_pipeline` covers actual WGPU
  handle reuse and stable shader/layout/pipeline cache counts.
- `optimization_batch_20260826br_ibl_pipeline_fast_hit_precedes_component_cache_probes` locks the
  complete-key early return and removal of the duplicate compute-map probe.
- `optimization_batch_20260826br_ibl_pipeline_fast_hit_p95` reports paired release P50/P95 samples
  and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Runtime09F1 still owns environment bake scheduling, PMREM/SH correctness, persistence, reflection
probe integration, and product GPU evidence. This slice only converges stable IBL bake pipeline
lookup.
