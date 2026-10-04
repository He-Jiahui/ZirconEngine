---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/169/2026-09-20-navigation-vertex-projection-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
tests:
  - tools/tests/test_runtime862_navigation_vertex_projection_capacity_performance_contract.py
  - tools/tests/test_runtime08d_borrowed_polygon_indices_performance_contract.py
---

# Runtime862 Navigation Polygon Vertex Projection Capacity

## Completion entry

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime baked polygon vertex projection | Reserve the validated index-slice bound before filtering vertex references, preserving invalid-index filtering, source order, duplicate handling, bounds, and center semantics. | Intentional RED/GREEN source-model contract `4/4`; lower dense-capacity/order regression and ignored `RUNTIME862_NAVIGATION_VERTEX_PROJECTION_CAPACITY_BENCH_V1` marker are wired. The deterministic 4,096-index model changes modeled legacy growth `11→0`; the original combined navigation batch passes `35/35` in `0.062s`, and the expanded batch including the repaired Runtime08d borrowed-index contract passes `38/38` in `0.139s`, with zero failures/errors/skips. Managed Cargo/Release, allocator, and navigation product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

The 2026-09-21 rerun of the same one-process non-tooling loader again passes
`4104/4104` across `970` files in `274.515s`, with zero failures, errors, load
errors, or skips. Fixture Cargo strings are not managed acceptance evidence.
The focused Runtime08d/861/862 navigation batch also passes `38/38` in
`0.021s`, with zero failures, errors, or skips.

## Scope boundary

This slice changes only the temporary vertex projection allocation shape. It
does not change navmesh validation, edge-key generation, sort/deduplication,
bounds, center, spatial indexing, query authority, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/navigation/runtime/baked_mesh.rs` | `0ECB4B874765D5CE51C88283126A54A497B81D63D423DBD5213F6D680493E558` |
| `tools/tests/test_runtime862_navigation_vertex_projection_capacity_performance_contract.py` | `175115A2729E03E86026ADF90844EA71B2561AF4F1ED6748EF70BBE8CCDC3D16` |
| `tools/tests/test_runtime08d_borrowed_polygon_indices_performance_contract.py` | `2D76031EF2A617942501FFF2B45C1512897AEE4E7E4F80C40D4E8F7A9F3490EC` |

## Managed gate

No managed Windows Cargo/Release command is started locally and coordinator
status is not polled. Keep this entry `implemented_pending_validation` until
the combined owner-attributed Windows Release lane proves current-source
compilation, lower-test reachability, allocator behavior, and navigation
product p50/p95/p99 evidence.
