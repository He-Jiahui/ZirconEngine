---
title: Runtime11C Auto Text Recency Sort Projection
category: zircon_runtime
report_id: Runtime11C-auto-text-recency-sort-projection-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C Auto Text Recency Sort Projection

## Scope

This slice reduces CPU work when the auto text raster-route recency queue is compacted. It supports
Runtime11C's stable text-route cache and P1-7 hot-path allocation/comparison work. It does not claim
ordered draw-op convergence, persistent text artifacts, a shared font/glyph owner, or zero-work
stable frames.

## Change

- Project each live route once to `((last_seen_frame, recency_token), &identity)` before sorting.
- Sort the cached scalar projection without looking the identity up in the route `HashMap` from the
  comparison key function.
- Clone each live identity only after the final order is known, preserving the owned recency queue
  and its lifetime boundary.
- Preserve the existing stable `(last_seen_frame, recency_token)` ordering and token values.

## Deterministic Performance Evidence

| 4,096 live routes | Before | After | Reduction |
|---|---:|---:|---:|
| Sort-key `HashMap` lookup path | active during sort | absent | removed |
| Sort key projection | recomputed through identity lookup | computed once per route | bounded to N |
| Final owned identity clones | 4,096 | 4,096 | unchanged |
| Recency ordering | `(frame, token)` stable sort | `(frame, token)` stable sort | unchanged |

The ignored release gate alternates 17 legacy lookup-sort and cached-projection samples. It emits
`RUNTIME11C_AUTO_TEXT_RECENCY_SORT_PROJECTION_BENCH_V1`; acceptance requires projected P95 to be at
most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826h_runtime11c_recency_projection_preserves_frame_token_order` covers
  frame/token ordering and identity/token association.
- `optimization_batch_20260826h_runtime11c_recency_sort_uses_cached_projection` rejects comparator
  lookup regression and requires the cached scalar projection.
- `optimization_batch_20260826h_runtime11c_recency_sort_projection_performance_evidence` emits both
  P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting with child traversal disabled, scoped diff checks, and source
  contracts must pass before managed validation submission.

## Remaining Parent-plan Work

Auto text routing still owns a private cache and compacts only after queue growth. Stable text UI
still rebuilds broader prepare artifacts, renderer submission remains split by text backend, and
cross-route cache/budget/device-generation convergence remains open.
