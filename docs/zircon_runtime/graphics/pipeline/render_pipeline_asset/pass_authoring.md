---
related_code:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/tests/pipeline_overlay_order.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/types.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 continue Plan 01 compile.rs modularization
tests:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_preserves_renderer_stage_for_each_graph_pass
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_preserves_compute_workload_from_feature_descriptor
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_keeps_split_postprocess_passes_before_exposure_when_they_do_not_sample_exposure
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_routes_output_transfer_through_smaa_terminal_input
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-pass-authoring-0617 --message-format short --color never
  - cargo test -p zircon_runtime --lib compile_keeps_split_postprocess_passes_before_exposure_when_they_do_not_sample_exposure --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-required-external-0618
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_forward_plus_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_deferred_pipeline_compiles_expected_stage_order_and_passes
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-dynamic-scene-asset-0619 --message-format short --color never
  - D:\cargo-targets\zircon-runtime-dynamic-scene-asset-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe graphics::tests::pipeline_compile::default_forward_plus_pipeline_compiles_expected_stage_order_and_passes --exact --test-threads=1 --nocapture
  - D:\cargo-targets\zircon-runtime-dynamic-scene-asset-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe graphics::tests::pipeline_compile::default_deferred_pipeline_compiles_expected_stage_order_and_passes --exact --test-threads=1 --nocapture
  - D:\cargo-targets\zircon-runtime-dynamic-scene-asset-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe graphics::pipeline::render_pipeline_asset::compile_tests::compile_routes_output_transfer_through_fxaa_terminal_input --exact --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib pipeline_overlay_order --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-texture-linear-product-0620 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib rendering_plugin_default_features_restore_legacy --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-texture-linear-product-0620 --message-format short --color never -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Pipeline Pass Authoring

## Purpose

`render_pipeline_asset/pass_authoring.rs` owns the compile-time lowering from filtered `RenderFeaturePassDescriptor` rows into a concrete `RenderGraphBuilder`. It creates graph handles, adds passes, writes pass metadata, maps declared resource accesses to builder calls, and returns the compiled graph plus `CompiledRenderPipelinePassStage` rows to `compile.rs`.

This keeps `compile.rs` as the orchestration boundary for validation, feature enablement, extract-section collection, capability collection, history binding collection, and frame-extract-dependent core particle descriptor insertion.

## Related Files

- `compile.rs` calls `author_render_graph(...)` after descriptor filtering and validation are complete.
- `descriptor_filtering.rs` decides which feature pass descriptors survive and which post-process resources are rerouted before this module sees them.
- `graph_resources.rs` decides whether each logical resource name is transient texture, transient buffer, or external import.
- `resource_descriptors.rs` supplies concrete `TextureDesc` and `BufferDesc` values for transient graph resources.
- `RenderGraphBuilder` remains the only module that owns final graph compilation, dependency inference, culling, and lifetime derivation.

## Behavior Model

Authoring happens in two phases.

First, `author_graph_resources(...)` scans the resource plan from `pipeline_graph_resources(...)` and allocates the matching logical graph handle for each resource name. Texture resources call `RenderGraphBuilder::create_texture(...)`, buffer resources call `create_buffer(...)`, and external resources call `import_external_resource_with_binding(...)` so required/report-only texture or buffer metadata reaches compiled lifetimes.

Second, `author_graph_passes(...)` walks renderer stages in asset order and asks the validation layer for pass descriptors belonging to each stage. Each pass receives:

- a graph pass name and executor id;
- the resolved queue lane from `RenderPipelineCompileOptions::resolve_queue(...)`, while preserving the descriptor's declared queue for diagnostics;
- pass flags and optional compute workload metadata;
- explicit resource read/write declarations mapped to the correct texture, buffer, or external graph handle;
- a dependency on the previous authored pass to preserve the existing serial SRP stage order.

Post-process pass rows pass through `ordered_stage_pass_descriptors(...)` before graph authoring. That helper keeps the validation-layer descriptor filtering as the source of truth, then normalizes Bloom extract ahead of the uber post-process pass when both descriptors are present so the authored graph does not depend on asset declaration order for that specific post-process dependency.

## Resource Access Rules

Read declarations forward directly to `read_texture`, `read_buffer`, or `read_external` after consulting the graph resource plan. A descriptor that names a texture resource later planned as external reads the external handle instead of inventing a transient texture.

Texture writes use descriptor attachment ops when provided. If no attachment ops are specified, the first transient texture write uses `clear_store()` and later writes to the same transient logical texture use `load_store()`. Storage writes use the storage path and do not attach load/store ops. External attachment writes default to `load_store()` when the descriptor does not provide explicit ops, preserving the existing external-target policy.

Buffer writes are direct storage-like graph writes for transient buffers or external writes for imported resources. If the resource plan says a declared texture is actually a buffer, or a declared buffer is actually a texture, the code keeps the same unreachable invariant used by the previous inline compile loop because `graph_resources.rs` should already reject conflicting declarations.

## Design And Rationale

The split follows the RDG/SRP boundary used in the render architecture plan: feature descriptors describe pass intent, resource planning decides logical ownership, descriptor sizing decides backend shape, and pass authoring is the final compile-time declaration step. It is not an execution module and does not bind WGPU resources.

The module returns only `AuthoredRenderGraph` instead of exposing the builder or handle maps. That keeps handle ownership local to graph authoring and prevents future compile orchestration code from bypassing resource planning.

## Test Coverage

Existing compile source-contract tests exercise this module through public pipeline compilation. Stage preservation verifies `CompiledRenderPipelinePassStage` output, compute workload preservation verifies pass metadata survives authoring, and terminal post-process routing tests verify authored resource declarations match the filtered descriptors.

The 2026-06-18 split post-process exposure regression covers descriptor-to-graph read lowering for dependency order. `compile_keeps_split_postprocess_passes_before_exposure_when_they_do_not_sample_exposure` asserts that DoF and motion-blur split passes do not inherit a false `EXPOSURE_CURRENT` read, while exposure resolve and uber still expose the real dependency edge.

Most broader pass-authoring focused lib-tests remain deferred to the milestone testing stage. The split post-process exposure regression listed above has been run directly, and the scoped `zircon_runtime --features core-min` Cargo check listed in the header remains the lightweight implementation gate for the original extraction.

On 2026-06-19, the Plan 09 texture-target Base+Overlay product guard first exposed a lower shared authoring bug: default post-process descriptors could author `post.uber` before `post.bloom-extract`, so a graph compile saw `post.uber` read `bloom-texture` before any producer wrote it. `ordered_stage_pass_descriptors(...)` now keeps descriptor filtering as the source of truth, but normalizes that current Bloom producer ahead of the uber consumer for `RenderPassStage::PostProcess`. After the dynamic-scene test fixture blocker was repaired in its owner path, the warmed lib-test binary passed the default Forward+ and Deferred pass-order exact tests listed in the header; the same binary also passed the FXAA terminal routing contract.

On 2026-06-20, the Plan 09 UI graph-tail ordering slice moved default Forward+ and Deferred 3D pipeline assets so `RenderPassStage::Overlay` and `RenderPassStage::Debug` execute before `RenderPassStage::Ui`. The compiled pass order now places `overlay-gizmo` before terminal `runtime-ui`, matching the screen-space UI graph-tail contract. `pipeline_overlay_order.rs` locks the contract for both default 3D pipelines, while the existing default Forward+/Deferred pass-order snapshots cover the full pass list. The pluginized legacy pass-order tests also pass after their test fixture declares `FINAL_COMPOSITED` as a graph texture instead of an external output when builtin FXAA reads it. A follow-up runtime execution regression now verifies late graph stages execute in compiled pipeline order, preserving Core2D `Ui`-before-`Overlay` while default 3D pipelines run `Overlay`/`Debug` before `Ui`.

## Open Issues Or Follow-up

Remaining RG-M1 work is behavior work: non-HZB/non-shadow-atlas executor-owned External actual binding, resource lifetime validation closure, broader post-process order regression tests beyond the focused default Forward+/Deferred checks, and the separate `render_graph_execution_resources.rs` transient materialization structure split.
