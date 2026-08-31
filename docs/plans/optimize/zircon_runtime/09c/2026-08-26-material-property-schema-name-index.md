---
title: Runtime09c Material Property Schema Rescan Elision
category: zircon_runtime
report_id: Runtime09c-material-property-schema-rescan-elision-2026-08-26
date: 2026-08-26
session_id: root-runtime09c-three-task-material-index-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09c Material Property Schema Rescan Elision

## Scope

This slice removes the repeated full shader-property-schema scan from material render-property
projection. Declared properties, defaults, invalid typed overrides, undeclared string fallback
properties, sorted output, and material-owned property filtering remain unchanged. It advances the
Runtime09c material/shader projection hot path without claiming completion of shader compilation,
pipeline caching, descriptor binding, or renderer product gates.

## Change

- Keep the first schema pass as the sole declared-property projection owner.
- Use the projected value map to reject already-handled string properties in the fallback pass.
- Remove the redundant schema membership scan: every string override is projected by the first
  pass regardless of declared kind, while non-string values cannot enter the string fallback.
- Add split behavior, deterministic work, and ignored release P95 tests.

## Deterministic Performance Evidence

| 4,096 schema properties and 4,096 undeclared string overrides | Before | After |
|---|---:|---:|
| Pairwise schema-name comparisons per projection | 16,777,216 | 0 |
| Fallback output-map membership probes | 4,096 | 4,096 |
| Temporary schema membership indexes | 0 | 0 |
| Output ordering or value changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME09C_MATERIAL_PROPERTY_SCHEMA_RESCAN_BENCH_V1`. Acceptance requires optimized projection
P95 to be at least 80% below the legacy repeated schema scan. Exact Windows timings remain pending
the coordinator run.

## Acceptance

- `runtime09c_batch_material_property_schema_rescan_preserves_projection` compares
  legacy and optimized output across declared valid, declared invalid, declared string, unknown
  string, and unknown non-string overrides.
- `runtime09c_batch_material_property_schema_rescan_eliminates_pairwise_work`
  requires 16,777,216 legacy comparisons and rejects schema access in the fallback loop.
- `runtime09c_batch_material_property_schema_rescan_p95` reports paired release
  P50/P95 samples and enforces the 80% P95 reduction gate.
- The managed `runtime09c_batch_` release gate covers this task, material-option value hashing,
  and shading-token hashing in one Cargo invocation: 3 source contracts, 9 Rust tests, and 3
  performance rows. Dynamic marker values, integration commit, and WeCom delivery remain
  coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime09c still owns shader variants, PSO keys, compiler diagnostics, pipeline-layout admission,
descriptor binding, persistent caches, reload invalidation, and renderer-scale evidence. This slice
only converges material property projection.
