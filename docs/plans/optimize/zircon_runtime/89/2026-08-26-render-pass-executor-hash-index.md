---
title: Runtime89 Render Pass Executor Hash Index
category: zircon_runtime
report_id: Runtime89-render-pass-executor-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime89 Render Pass Executor Hash Index

## Scope

This slice replaces the render-pass executor registry owner with `HashMap`. Per-pass execution,
parallel-recording policy checks, compiled-pipeline validation, registration, and removal now
resolve executor IDs through expected constant-time lookup.

The registry exposes no executor iterator or ordered snapshot. Compiled graph order remains the
execution authority, registration generation and validation-cache invalidation are unchanged, and
`RenderPassExecutorId` retains borrowed `str` lookup without allocating on hits.

## Performance Workload

The release workload fills 4,096 long shared-prefix executor IDs and performs 4,096 stable borrowed
string hits for the final executor ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered executor lookups | 4,096 | 0 |
| Hash executor lookups | 0 | 4,096 |
| Executor-order policy changes | 0 | 0 |
| Allocations on executor hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME89_RENDER_PASS_EXECUTOR_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ca_render_pass_executor_hash_index_preserves_registration` covers
  independent registration, borrowed lookup, and removal.
- `optimization_batch_20260826ca_render_pass_executor_hash_index_has_no_order_contract` locks the
  unordered owner and absence of registry traversal.
- `optimization_batch_20260826ca_render_pass_executor_hash_index_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime89 still owns compiled execution packets, resource lifetime, barriers, queue scheduling,
transient aliasing, GPU evidence, and product integration. This slice only converges executor-ID
lookup.
