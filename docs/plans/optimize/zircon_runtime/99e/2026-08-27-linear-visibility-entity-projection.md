---
title: Runtime99E Linear Visibility Entity Projection
category: zircon_runtime
report_id: Runtime99E-linear-visibility-entity-projection-2026-08-27
date: 2026-08-27
session_id: root-runtime99e-linear-visibility-entity-projection-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99E Linear Visibility Entity Projection

## Scope

This slice reduces allocation and ordered-tree work while projecting scene mesh, Sprite, and
particle renderables into the visibility membership lists. It advances Runtime99E P1-13/P1-14
scale work without changing culling authority, bounds, phase queues, or public output order.

## Change

- Replace three independent `BTreeSet` construction passes with one pass over renderables.
- Append entity IDs into preallocated renderable/static/dynamic vectors.
- Restore the existing sorted-unique public contract with `sort_unstable` and `dedup`.
- Preserve the case where one entity appears in both mobility groups while remaining unique in
  each public list.

The projection remains O(N log N) because public vectors are ordered. It removes tree-node
admissions and reduces source traversal from three passes to one; a future dense membership
artifact can remove sorting when consumers accept generation-qualified slots.

## Deterministic Performance Evidence

Independent optimized Rust 1.94.1 model, 100,000 renderables, 80,000 entity IDs with duplicates,
mixed mobility, 2 repetitions, and 21 alternating samples:

| Metric | Three `BTreeSet` passes | Linear vectors | Reduction |
|---|---:|---:|---:|
| allocations | 430 | 6 | 98.60% |
| allocated bytes | 11,161,600 | 4,800,000 | 56.99% |
| P50 | 12,438,200 ns | 6,134,000 ns | 50.68% |
| P95 | 32,628,100 ns | 11,540,800 ns | 64.63% |

The executable gate requires at least 98% fewer allocations, at least 30% fewer allocated bytes,
and at least 35% lower P95. The stable checksum is `6368421876654401508`.

## Acceptance

- The Rust regression covers sorted uniqueness, duplicate entity IDs, and cross-mobility
  membership preservation.
- The Python source contract rejects `BTreeSet`, requires one renderable pass, and requires all
  three output vectors to use the shared sort/dedup finalizer.
- The independent model emits `RUNTIME99E_LINEAR_VISIBILITY_ENTITY_PROJECTION_MODEL_V1` and
  enforces allocation, byte, and P95 targets.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, focused Rust behavior, relevant scene
  render-extract regressions, source contracts, and model execution are submitted in one batch.

## Remaining Parent-Plan Work

World collection still clones and sorts complete Sprite and mesh snapshots. Visibility still lacks
2D bounds and camera-rectangle culling, and the empty-phase fallback remains unsafe. Persistent
Canvas2D extract pages, bounds-backed visibility, phase readiness, and GPU instance submission
remain open Runtime99E work.
