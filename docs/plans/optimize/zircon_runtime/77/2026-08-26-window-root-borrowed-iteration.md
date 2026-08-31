---
title: Runtime77 Window Root Borrowed Iteration
category: zircon_runtime
report_id: Runtime77-window-root-borrowed-iteration-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Window Root Borrowed Iteration

## Scope

This slice removes full root-list snapshots from window resize, scale-factor, and redraw dirty
propagation. Root order, dirty domains, missing-node errors, invalidation publication, and window
event diagnostics remain unchanged.

## Change

- Capture the stable root count and copy one `UiNodeId` by index before each mutable dirty update.
- Avoid cloning the retained root `Vec` solely to satisfy the temporary immutable/mutable borrow
  boundary.
- Rely on the existing `mark_node_dirty` contract, which updates nodes and invalidation state but
  does not mutate the root list.

## Deterministic Performance Evidence

| 16,384 roots, 32 enumeration probes per sample | Before | After |
|---|---:|---:|
| Root-list snapshots per sample | 32 | 0 |
| Root IDs copied into temporary lists | 524,288 | 0 |
| Root IDs visited | 524,288 | 524,288 |
| Enumeration allocation complexity | `O(N)` per event | `O(1)` per event |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME77_WINDOW_ROOT_BORROWED_ITERATION_BENCH_V1`. Acceptance requires borrowed root iteration
P95 to be at least 30% below root-snapshot P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826aq_window_root_iteration_marks_every_root` uses a real multi-root
  `UiSurface` and verifies every root plus the invalidation count.
- `optimization_batch_20260826aq_window_root_iteration_avoids_root_snapshot` rejects root cloning,
  collection, and `to_vec` while requiring stable indexed reads.
- `optimization_batch_20260826aq_window_root_borrowed_iteration_p95` reports paired P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime77 still owns qualified window/input identity, lifecycle teardown, transactional dispatch,
navigation indexing, route scratch, backpressure, and product-scale performance receipts. This
slice only converges window-event root dirty enumeration.
