---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09f1/2026-08-26-ibl-bake-pipeline-fast-hit.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/972-runtime09d-to-09h1-completion-list.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache/fast_hit_tests.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache/fast_hit_tests.rs
---

# Runtime968 · IBL bake pipeline fast hit

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09F1 compute-pipeline cache | Stable PMREM/SH bake requests probe the complete pipeline-key map first and return the cached WGPU handle before shader-module/layout component probes. Miss behavior and handle ownership are unchanged; newly created pipelines are inserted and returned directly. | The focused tests cover real handle reuse, stable cache counts, early-return ordering, and the ignored paired Release benchmark. Stable-hit counted probes fall from `12,288` to `4,096` (`66.7%`); the P95 gate requires at least 50% reduction. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs` | `0691645EF7075F94CE007A2E2B5C3407E24AB92B2A8C9F55675609548B8D15BA` |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache/fast_hit_tests.rs` | `D29E9C27FEAED4C31923A8AD0D1795A94EDBCC969FD626E40B3AEACD29BF5389` |

## Validation handoff

The focused source/tests and `RUNTIME09F1_IBL_PIPELINE_FAST_HIT_BENCH_V1`
marker are included in the grouped Runtime package validation. Exact managed
Windows P50/P95 and product GPU evidence remain pending.
