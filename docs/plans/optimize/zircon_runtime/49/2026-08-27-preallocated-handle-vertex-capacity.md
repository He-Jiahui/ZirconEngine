---
title: Runtime49 Preallocated Handle Vertex Capacity
category: zircon_runtime
report_id: Runtime49-preallocated-handle-vertex-capacity-2026-08-27
date: 2026-08-27
session_id: root-runtime49-preallocated-handle-vertex-capacity-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime49 Preallocated Handle Vertex Capacity

## Scope

`build_handle_vertices` previously appended every handle line, arrow head, ring, scale cross, and
center anchor to an empty `Vec`. A representative 8,192-element overlay grew the output allocation
17 times and repeatedly copied the already-built vertex prefix.

The line-geometry helpers now export their non-degenerate vertex capacities. The ring segment count
also owns its derived capacity, so the builder estimate and tessellation loop cannot use different
segment constants. The handle builder makes one read-only pass over the element topology, sums the
shared capacities, and allocates the output once before preserving the existing append order.
Degenerate arrow heads and rings still emit no helper vertices; they may leave unused capacity but
do not change output geometry, camera basis use, ordering, color, or empty-frame behavior.

This is a bounded current-path improvement. It does not close the parent plan's retained geometry,
visibility, per-view budget, persistent GPU arena, dirty upload, or overlay pass consolidation work.

## Performance Evidence

The isolated release model mirrors the production mix of axis lines, 48-segment rings, axis scales,
and center anchors with a 28-byte line vertex payload. It runs 31 alternating sample pairs, eight
rounds per sample, over 8,192 elements and 229,376 output vertices. The model was compiled with
`rustc -O` on Windows.

| Metric | Growing `Vec` | Preallocated capacity | Change |
|---|---:|---:|---:|
| Allocator calls per build | 17 | 1 | -94.12% |
| Cumulative requested bytes per build | 14,679,952 | 6,422,528 | -56.25% |
| P50 for 8 rounds | 56,727,900 ns | 37,231,400 ns | -34.368% |
| P95 for 8 rounds | 90,764,900 ns | 56,487,100 ns | -37.765% |

Model source:
`.codex/state/session-coordinator/runtime49-preallocated-handle-vertex-capacity-model.rs`.

## Contracts And Validation

- `tools/tests/test_runtime49_preallocated_handle_vertex_capacity_performance_contract.py` locks
  the shared topology capacities, builder preallocation fold, and absence of the growing output
  vector.
- Rust behavior tests require each arrow, cross, and ring helper's declared capacity to match its
  non-degenerate emitted vertex count; the handle capacity test covers all four element variants.
- Local source-contract result: 3 tests passed.
- Local `rustfmt --edition 2021 --check` passed for all six production Rust files.
- The release model passed its allocation, requested-byte, P50, and P95 gates.
- Cargo compilation and focused Rust behavior tests remain pending in the managed asynchronous
  coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime49 still owns the canonical gizmo producer and category registry, immediate and retained
lifetime, geometry correctness, view filtering, deterministic budgets, diagnostics, persistent GPU
storage, domain cutovers, and product-scale GPU qualification. This slice only removes repeated CPU
vertex-vector growth on the existing transform-handle path.
