---
title: Runtime09B Indexed Virtual Geometry Execution Projection
category: zircon_runtime
report_id: Runtime09B-indexed-virtual-geometry-execution-projection-2026-08-27
date: 2026-08-27
session_id: root-runtime09b-dirty-proportional-static-index-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Indexed Virtual Geometry Execution Projection

## Scope

This slice removes nested cluster and instance scans from the virtual-geometry debug snapshot
execution projection. It advances Runtime09B P1-6 observability cost without changing draw-segment
admission, page state, submission ordering, public snapshot fields, or GPU execution.

## Change

- Build one frame-local cluster-to-first-instance projection, preserving overlapping-range first
  match semantics.
- Build one stable-key-to-sorted-unique-cluster-id projection for ordinal resolution.
- Build one composite cluster hash index and retain the first source cluster for duplicate keys.
- Reuse the same lookup for execution-segment instance resolution and selected-cluster expansion.
- Preserve zero stable-key legacy derivation and missing-row behavior.

The projection changes from repeated O(S * C * I) scans plus per-candidate cluster-id allocation and
sorting to O(C + covered instance spans + S) expected work with frame-local O(C) index storage.

## Deterministic Performance Evidence

Independent optimized Rust 1.94.1 model, 64 instances, 1,024 clusters, 1,024 reversed execution
segments, and 31 alternating samples. Three complete runs produced the same checksum and allocation
counts. The canonical third run was:

| Metric | Nested scan | Shared indexed projection | Reduction |
|---|---:|---:|---:|
| allocations | 2,561 | 73 | 97.15% |
| allocated bytes | 180,224 | 121,068 | 32.82% |
| P50 | 2,713,400 ns | 356,200 ns | 86.87% |
| P95 | 6,414,900 ns | 1,238,600 ns | 80.69% |

Across all three runs, P50 reduction was at least 84.79% and P95 reduction was at least 51.82%.
The stable checksum was `0`. The in-repository ignored release benchmark includes lookup
construction in every optimized sample and requires indexed P95 to be at most 60% of legacy P95.

## Acceptance

- Rust regressions cover overlapping instance ranges, duplicate cluster first-match behavior,
  sorted/deduplicated ordinal identity, and legacy zero-key derivation.
- The Python contract requires one root lookup construction reused by both projection stages and
  rejects restoration of the former nested linear helpers.
- The release benchmark emits
  `RUNTIME09B_INDEXED_VIRTUAL_GEOMETRY_EXECUTION_BENCH_V1`, validates equivalent results, and
  enforces the 60% P95 ratio gate.
- Exact-file Rust 1.94.1 formatting, source contracts, the focused Rust test filter, the independent
  allocation model, and scoped diff checks are submitted together in one managed batch.

## Remaining Parent-Plan Work

This index is scoped to debug snapshot construction and is rebuilt per requested snapshot. The
renderer still needs persistent GPU-scene ownership, generation-qualified handles, bounded debug
capture admission, and production CPU/GPU trace evidence at 100K and 1M primitive scales.
