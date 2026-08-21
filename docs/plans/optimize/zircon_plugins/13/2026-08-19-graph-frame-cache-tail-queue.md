# Plugins13 Graph Frame Cache Tail-Queue Optimization Record

- Date: 2026-08-19
- Owner: `plugins13-graph-frame-cache-tail-queue-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md`, NANI-P1-027
- Status: implementation, cross-module compile repair, and 21-pair measurement repair complete; managed revalidation pending

## Problem

The per-frame graph evaluation cache retained at most 256 entries in a `Vec`.
Every insertion after saturation evicted the oldest entry with `remove(0)`,
shifting 255 complete cache records before evaluating the next graph.

## Change

- The cache owner now stores entries in `VecDeque`.
- Saturated FIFO eviction uses O(1) `pop_front`; insertion uses `push_back`.
- Linear key lookup, parameter equality, capacity, frame reset, and FIFO
  replacement semantics remain unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 100,000 inserts at capacity 256 | 25,434,720 entry moves | 0 entry moves | 100% |
| One saturated eviction | 255 moves, O(capacity) | 0 moves, O(1) | one complexity class |
| Cache residency | 256 entries | 256 entries | unchanged |

## Acceptance

- `graph_evaluation_frame_cache_preserves_fifo_eviction_order` inserts 257
  unique graphs and proves the oldest entry retires while the newest 256 stay
  ordered.
- `graph_evaluation_frame_cache_source_uses_tail_queue_eviction` rejects a
  restored `Vec` or `remove(0)` in the production owner.
- `graph_evaluation_frame_cache_tail_queue_release_benchmark_evidence` emits
  21 paired, alternating release timing samples for 100,000 saturated inserts
  and computes nearest-rank P95.
- Timing gate: tail-queue P95 must be no more than 25% of legacy P95.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regression and release P50/P95: pending a replacement combined Windows
  coordinator batch after the main source baseline integrates; no per-task
  Cargo command is used.

## Managed Validation Evidence

- Validation-copy job `fbde9ed72d64434cb23158595d834ed3` reached the
  `graph_frame_cache_regressions` stage and failed before the owned tests or
  benchmark ran.
- The compiler reported two `E0308` errors because the physics module factory
  now receives `&CoreWeak`, while `DefaultPhysicsManager::new` and
  `attach_core` still required `&CoreHandle`.
- Static follow-up found the same stale constructor contract in the animation
  manager that owns this optimization. Both managers now retain the borrowed
  weak runtime handle directly and upgrade it only while loading configuration.
- The factory call sites and the no-strong-reference-cycle source contract are
  unchanged. Exact-file Rustfmt, scoped `git diff --check`, and weak-handle
  source assertions pass; Cargo proof remains assigned to the next batch.

## Remaining Scope

The cache still performs linear key lookup and clones the complete parameter
map into each entry. It has no byte budget, lease-aware retirement, or
hit/miss/eviction telemetry. This record closes only front-removal complexity;
NANI-P1-027 and product-scale G29 remain open.
