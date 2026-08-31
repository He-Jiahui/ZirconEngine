# Runtime93 Optional UV1 Lazy Materialization Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plans: Runtime93 MESH93-P1-06/P1-30 and Runtime97 lightmap UV engineering
- Status: implementation and release gate authored; Runtime93 three-task managed batch pending

## Problem

`MeshAsset::from_model_primitive` always collected one UV1 value per vertex into
a new `Vec<[f32; 2]>`, then scanned that vector to decide whether the optional
attribute should exist. Meshes whose vertices all carried the default UV1 value
discarded the full allocation and every copied element immediately.

## Change

- Add one `optional_uv1_values` authority for optional channel presence.
- Scan borrowed `MeshVertex` data first and return `None` without allocating when
  every UV1 value is the default.
- Materialize the exact UV1 vector only when at least one authored/non-default
  value requires the attribute to be preserved.
- Preserve channel presence semantics, vertex order, default entries surrounding
  authored values, attribute type, and all other mesh conversion fields.

## Deterministic Performance Evidence

| 262,144-vertex mesh without UV1 | Before | After | Reduction |
|---|---:|---:|---:|
| Temporary UV1 vectors | 1 | 0 | 100% removed |
| Copied UV1 values | 262,144 | 0 | 100% removed |
| Borrowed presence predicates | 262,144 | 262,144 | unchanged |
| Serialized UV1 attribute | absent | absent | unchanged |

The ignored release gate runs 17 alternating allocate-then-scan/borrowed-scan
sample pairs. Acceptance requires borrowed-scan nearest-rank P95 to be at most
60% of legacy P95, a minimum 40% reduction. Exact Windows timings remain
pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826e_runtime93_optional_uv1_scans_before_materializing`
  locks borrowed presence detection before materialization.
- `optimization_batch_20260826e_runtime93_optional_uv1_preserves_presence_and_values`
  covers both absent and mixed default/authored UV1 inputs.
- `optimization_batch_20260826e_runtime93_optional_uv1_performance_evidence`
  emits `RUNTIME93_OPTIONAL_UV1_LAZY_MATERIALIZATION_BENCH_V1`, raw samples,
  vertex count, allocation/copy counts, and the 40% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt with child traversal disabled, source contracts,
  five non-ignored Runtime93 behavior regressions, two ignored release gates, the
  locator allocation/timing model, and scoped diff checks are submitted as one
  coordinator-managed three-task batch.

## Remaining Plan Work

This slice does not close Runtime93. Canonical section/LOD artifacts, source
channel provenance, UV validation/generation, bulk retention policy, async
prepare/residency, bounds/instancing, skin/morph pipelines, collision cook, and
10K/100K product qualification remain open.
