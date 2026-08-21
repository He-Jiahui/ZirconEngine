# Runtime89 Compiled Pass Identity

Plan: docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md","docs/plans/optimize/zircon_runtime/89/2026-08-21-compiled-pass-identity.md","docs/zircon_runtime/graphics/pipeline/render_pipeline_asset/pass_authoring.md","docs/zircon_runtime/render_graph/builder.md","zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs","zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/tests.rs","zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs","zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs","zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs","zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/sprite_stage_selection.rs","zircon_runtime/src/render_graph/builder/compile.rs","zircon_runtime/src/render_graph/error.rs","zircon_runtime/src/render_graph/graph.rs","zircon_runtime/src/render_graph/tests/ordering.rs"]

- Date: 2026-08-21
- Integration owner: `optimize-runtime73-runtime81-runtime89-batch-m3-r1-01a00797-20260822`
- Former integration owner: `optimize-runtime81-shaping-budget-m0-r1-01a00797-20260821`
  (`cancelled` after grouped transfer fingerprint
  `2b9dfa137351b0771e7c7f4fa03b571010756adbbf0c8a60bde3ca394807aa29`)
- Original owner: `optimize-runtime89-compiled-pass-identity-m0-r1-01a00797-20260821`
  (`cancelled` after transfer fingerprint
  `5e948746a28a3ca7e1f5cee8af928dc387e495e0a9b8cac19cb3619f9175ddb3`)
- Source item: `RG89-P0-001` / M0 final pass namespace and identity slice
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Pipeline stage metadata retained only a pass name. Each stage execution then scanned the
compiled graph from the beginning to recover the first matching pass. This made product
execution quadratic in pass count and allowed a plugin pass with a generated IBL name to
shadow the real generated pass.

## Scope Delivered

- `RenderGraphBuilder::compile(...)` validates the final pass namespace and returns typed
  `RenderGraphError::DuplicatePassName` before dependency inference and index construction.
- `CompiledRenderPipelinePassStage` retains the builder-scoped `RenderPassId` beside its
  diagnostic name and stage.
- Feature authoring records the ID returned by `add_pass*`; IBL authoring consumes the IDs
  returned by `append_ibl_bake_artifact_graph_plan(...)`.
- Stage execution resolves the compiled index and pass through the existing ID hash index.
  Sprite-stage admission uses the same direct identity path. Both paths retain a defensive
  ID-to-name consistency check.
- The old per-entry `.find(|pass| pass.name == ...)` execution lookup is removed. The wider
  M6 execution-packet hard cut and removal of the separate stage vector remain future scope.

## Deterministic Performance Evidence

The release gate builds 2,048 pass identities outside the timed region, warms both paths once,
and runs 21 alternating legacy/optimized sample pairs. The raw marker retains both unsorted
21-sample nanosecond series, identifies the 11 legacy-first and 10 optimized-first pairs, and lets
the external validator independently recompute nearest-rank P95 and the reported ratio. One legacy
sample performs exactly 2,098,176 name comparisons
for ordered lookups; one optimized sample performs 2,048 ID-index lookups, a deterministic
99.90% reduction in lookup operations. Nearest-rank P95 uses rank 20 of 21 samples.

Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 25`, meaning direct identity
lookup must be at least 75% faster at P95. The managed validator emits one `PERF_RESULT` row
containing pass count, sample pairs, both P95 values, exact work counts, and ratio percentage
after validating the raw test marker. Actual
timing values remain pending; this record does not substitute structural counts for measured
release evidence.

## Fresh Testing Evidence

- Core duplicate pass names are rejected with the exact typed error.
- A plugin pass colliding with generated `env.ibl_irradiance_sh` fails during product compile.
- A full production authoring path containing one feature pass plus every PMREM, SH9, and IEM pass
  asserts the complete ordered stage-row ID/name bijection against the compiled graph.
- Rustfmt, scoped static checks, grouped Cargo tests, and the release performance gate are
  pending the shared coordinator batch. No Cargo or performance pass is claimed here.

## Review

Independent static review found no source behavior defect but required externally recomputable raw
timing samples and full-content production IBL coverage. Both evidence gaps are repaired, and
follow-up review found no Critical, Important, or Minor defect. Grouped Cargo validation remains
pending; no timing pass is claimed by this record.

## Remaining Scope

This slice closes only `RG89-P0-001` after validation. Sparse-resource admission
(`RG89-P0-002`), typed storage texture format validation (`RG89-P0-003`), full immutable
execution-packet convergence, backend barriers/queues, allocation lifetime, and Editor graph
diagnostics remain under later Runtime89 milestones and Runtime09A dependencies.
