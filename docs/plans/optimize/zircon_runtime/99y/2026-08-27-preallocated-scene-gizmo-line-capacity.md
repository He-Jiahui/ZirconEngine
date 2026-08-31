---
title: Runtime99Y Preallocated Scene Gizmo Line Capacity
category: zircon_runtime
report_id: Runtime99Y-preallocated-scene-gizmo-line-capacity-2026-08-27
date: 2026-08-27
session_id: root-runtime99y-preallocated-scene-gizmo-line-capacity-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99Y Preallocated Scene Gizmo Line Capacity

## Scope

`build_scene_gizmo_line_vertices` previously appended ordinary line segments, wire shapes, and
missing-texture icon fallbacks to an empty `Vec`. A representative 4,096-gizmo frame grew that
output allocation 18 times and repeatedly copied the already-built line vertex prefix.

The wire-shape and icon-fallback owners now expose capacity functions whose Rust behavior tests
invoke the real append helpers. The builder makes one read-only topology pass, counts ordinary
lines, wire shapes, and only icons without an atlas texture, then allocates the output once. The
second pass preserves existing gizmo, line, shape, and icon order. The atlas lookup is a pure
two-slot readiness check and is called once per icon in each pass. Degenerate arrows may leave four
unused vertices of capacity while retaining their existing two-vertex line output.

This is a bounded current-path improvement. It does not close the parent plan's stable compiled
artifact, persistent GPU arena, dirty upload, icon instancing, per-view visibility, deterministic
budget, or overlay pass consolidation work.

## Performance Evidence

The isolated release model mirrors 4,096 gizmos containing eight ordinary lines, two frustums, two
arrows, and two icons each with a 28-byte line vertex payload. It runs 31 alternating sample pairs
and eight rounds per sample. The model was compiled with `rustc -O` on Windows.

The mixed atlas/fallback workload emits 365,760 vertices:

| Metric | Growing `Vec` | Preallocated capacity | Change |
|---|---:|---:|---:|
| Allocator calls per build | 18 | 1 | -94.444% |
| Cumulative requested bytes per build | 29,360,016 | 10,241,280 | -65.118% |
| P50 for 8 rounds | 96,196,100 ns | 44,969,600 ns | -53.252% |
| P95 for 8 rounds | 162,400,100 ns | 62,761,600 ns | -61.354% |

The all-icons-textured boundary emits 311,296 vertices and no fallback icon lines. It still reduces
requested bytes by 70.312%, P50 from 83,575,700 ns to 40,322,600 ns (-51.753%), and P95 from
125,319,300 ns to 50,635,200 ns (-59.595%). This verifies that the extra pure atlas scan does not
erase the allocation benefit when every icon is already resident.

Model source:
`.codex/state/session-coordinator/runtime49-preallocated-scene-gizmo-line-capacity-model.rs`.

## Contracts And Validation

- `tools/tests/test_runtime99y_preallocated_scene_gizmo_line_capacity_performance_contract.py`
  locks the capacity prepass, missing-texture filter, shared helper capacity functions, and absence
  of the growing output vector.
- Rust behavior tests require non-degenerate frustum/arrow and both icon fallback capacities to
  match their real append output. A builder-level test covers no, partial, and complete icon atlas
  residency.
- Local source-contract result: 3 tests passed.
- Local `rustfmt --edition 2021 --check` passed for all four production Rust files.
- Both release-model atlas modes passed allocation, requested-byte, P50, and P95 gates.
- Cargo compilation and focused Rust behavior tests remain pending in the managed asynchronous
  coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime99Y still owns canonical producer/category integration, lifetime and generation, geometry
validation, view filtering, deterministic budget and receipts, diagnostics, persistent GPU storage,
domain cutovers, icon instancing, overlay pass consolidation, and product-scale GPU qualification.
This slice only removes repeated CPU line-vector growth on the existing scene-gizmo renderer path.
