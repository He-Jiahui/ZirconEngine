---
related_code:
  - zircon_runtime/src/core/framework/render/relevance.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/visibility/mod.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_relevance_entry.rs
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/graphics/visibility/culling/is_mesh_visible.rs
  - zircon_runtime/src/graphics/visibility/culling/mesh_bounds.rs
  - zircon_runtime/src/graphics/visibility/occlusion/mod.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_frame_visibility.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/relevance.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_relevance_entry.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/graphics/visibility/culling/is_mesh_visible.rs
  - zircon_runtime/src/graphics/visibility/culling/mod.rs
  - zircon_runtime/src/graphics/visibility/occlusion/mod.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_frame_visibility.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/depth_prepass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/opaque_base.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
plan_sources:
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/index.md
tests:
  - zircon_runtime/src/core/framework/render/relevance.rs::tests::primitive_relevance_tracks_material_layer_and_motion_policy
  - zircon_runtime/src/core/framework/render/relevance.rs::tests::primitive_relevance_keeps_shadow_eligibility_separate_from_main_view_layers
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs::tests::parallel_frustum_visibility_matches_serial_order_and_results
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_records_relevance_and_filters_main_view_layers
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_builds_shadow_view_independent_from_main_layers
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs::tests::frame_submission_context_exposes_view_visibility_by_key
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::update_visibility_stats_sums_per_view_culling_counts
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs::tests::processors_keep_shadow_candidate_when_main_view_layer_filters_mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs::tests::shadow_processor_respects_shadow_view_visibility
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_visibility_stats
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs::tests::hzb_builder_sizes_odd_viewport_to_half_power_of_two_chain
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs::tests::hzb_builder_keeps_one_pixel_viewports_valid
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs::tests::hzb_builder_reduce_passes_cover_tail_mips
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_describes_hzb_as_half_power_of_two_mip_chain
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_hzb_stats
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/core/framework/render/relevance.rs zircon_runtime/src/graphics/visibility/mod.rs zircon_runtime/src/graphics/visibility/declarations/mod.rs zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs zircon_runtime/src/graphics/visibility/declarations/visibility_relevance_entry.rs zircon_runtime/src/graphics/visibility/culling/mod.rs zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs zircon_runtime/src/graphics/visibility/context/from_extract_with_history/batching_result.rs zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain
doc_type: module-detail
---

# Visibility Module

## Purpose

`graphics::visibility` is the CPU visibility and culling bridge between render-frame extraction and the mesh/render planning layers. Plan 04 moves this module toward an InitViews-style pipeline: extract renderables, calculate primitive relevance once, run frustum culling, and later feed per-view visibility plus HZB occlusion into mesh command filtering and GPU-driven indirect execution.

This module remains WGPU-free. It consumes `RenderFrameExtract`, `ViewportCameraSnapshot`, `RenderLayerSet`, `GeometryPhaseInput`, and mesh snapshots from `core::framework::render`, then emits `VisibilityContext` data that render submission and planning code can inspect.

## Current VC-M1 Slice

The first VC-M1 code slice adds `PrimitiveRelevance` in `core/framework/render/relevance.rs`. It is a compact bitset describing how a render primitive participates in product phases:

- render-layer match for the active view
- main-view participation
- opaque, alpha-mask, or transparent material class
- depth-prepass eligibility
- shadow-caster eligibility
- deferred geometry eligibility for Core3d opaque-like primitives
- motion-vector candidate eligibility for dynamic opaque-like primitives

The relevance type intentionally separates main-view layer visibility from shadow eligibility. A mesh outside the current camera render layer is not relevant to opaque/alpha/transparent main-view phases or prepass, but an opaque-like mesh can still report shadow-caster eligibility for future independent shadow-view culling.

`VisibilityRelevanceEntry` stores the entity plus its `PrimitiveRelevance`. `VisibilityContext::primitive_relevance` keeps these entries beside the existing `visible_entities`, `culled_entities`, batches, history, and upload plans. This preserves old consumers while giving future per-phase command filtering a single relevance source instead of repeating alpha/shadow/motion decisions in each pass.

`FrameVisibility` and `ViewVisibilityContext` now provide the per-view container. The current implementation emits the main camera plus one `ShadowCascade { cascade: 0 }` view per extracted directional light. It already uses the final shape: a stable frame primitive index space (`entities`, `bounds`, `relevance`) plus view-local visible indices and `ViewCullingStats`. `VisibilityContext` keeps `frame_visibility` beside the legacy flat fields so later custom render-target camera slices can migrate consumers without breaking existing batching and history code.

## Culling Flow

`collect_batching_result(...)` now builds three lookups from the frame extract:

- mesh snapshots by entity
- geometry phase inputs by entity, to recover the extracted `RenderMaterialAlphaMode`
- visibility renderable entries, falling back to mesh snapshots when the extract does not provide explicit visibility inputs

It then builds a linear candidate array of `{ entity, VisibilityBounds }` and runs `mesh_frustum_visibility(...)` over that array. The helper uses a deterministic serial path for small scenes and a rayon `par_iter` path for larger scenes. The parallel path preserves input order in the collected result, so batch generation remains stable.

Final main-view visibility requires both `PrimitiveRelevance::main_view()` and a positive frustum result. This means camera `RenderLayerSet` filtering now participates in `visible_entities` and `visible_batches`. A layer-mismatched mesh is moved to `culled_entities`, but its relevance entry remains available for diagnostics and future non-main views.

`FrameVisibility::from_frame_views(...)` converts the same result into index-oriented view data. `ViewVisibilityContext::visible` stores `u32` indices into `FrameVisibility::entities`, not entity ids. Main-view stats count the original primitive input, layer-filtered primitives, frustum-culled primitives, occlusion-culled primitives, and final visible primitive count. Directional shadow views derive an orthographic light camera from the frame bounds, ignore main-camera layer visibility, and filter candidates through `PrimitiveRelevance::shadow_caster()` plus their own frustum result. Occlusion is fixed at zero until VC-M3 wires HZB.

`FrameVisibility` exposes view-key and entity-set helpers for consumers that are already moving away from the legacy flat fields. `main_view_visible_entity_set()` is now the source passed into Hybrid GI and Virtual Geometry visibility planning, while `shadow_views()` / `shadow_visible_entity_set()` give shadow consumers a single place to union directional shadow cascade results without scanning the raw view vector themselves.

`construct.rs` also derives `visible_batches`, `visible_instances`, `draw_commands`, and GPU instancing candidates from that same main-view entity set. `collect_batching_result.rs` no longer carries a separate `visible_batches` map, so the frame has one view-authoritative source for main-camera visibility instead of two parallel flat collections.

## Mesh Pass Consumption

`FrameSubmissionContext` now carries the computed `VisibilityContext` into both runtime-frame paths. `ViewportRenderFrame` stores the `FrameVisibility` sideband so the renderer can consume visibility without rebuilding it from scene data.

`build_mesh_draws(...)` maps frame primitive indices back to source entities, attaches `PrimitiveRelevance`, main-view visibility, and shadow-view visibility onto each `MeshDraw`, and then forwards those flags into `MeshBatchRef`. The mesh pass processors use that data as the pass-participation gate:

- depth prepass, opaque, alpha-mask, transparent, and motion-vector commands require main-view visibility and the matching relevance bit
- shadow commands require shadow-caster relevance and at least one shadow view containing the primitive
- if no directional shadow view exists, shadow submission falls back to `shadow_caster` relevance so the existing preview/default-shadow path still has valid caster candidates

The older queue profile still determines which material phase and pipeline variant a draw would use. Relevance now decides whether the draw participates in that phase for the current view.

`FrameSubmissionContext::view_visibility(key)` exposes per-view results to submit-time consumers. The Virtual Geometry debug node/cluster cull snapshot now reads the main camera through that accessor, so debug replay follows the same view authority as runtime visibility instead of reaching back to `frame_extract.view.camera` first.

## Visibility Diagnostics

`RenderStats` now exposes frame-level visibility counters derived from `FrameVisibility.views`:

- `last_visibility_view_count`
- `last_visibility_input_count`
- `last_visibility_layer_filtered_count`
- `last_visibility_frustum_culled_count`
- `last_visibility_occlusion_culled_count`
- `last_visibility_visible_count`

`update_base_stats(...)` sums the per-view `ViewCullingStats` rows each submitted frame. The current VC-M1 implementation therefore reports main-view plus directional shadow-view CPU culling work; occlusion remains zero until VC-M3 wires HZB/GPU occlusion into the same stats path.

`render_stats_store::product` records those fields under `render.visibility.*`, and the runtime diagnostics fixture asserts the same paths. This gives product diagnostics, devtools snapshots, and future tests a stable place to verify that relevance, layer filtering, frustum culling, and later occlusion are all using the per-view visibility authority.

## HZB Occlusion Foundation

VC-M2 introduces the shared HZB foundation that later GPU occlusion, SSR, and SSAO consumers will use instead of each feature owning private depth preparation. `graphics::visibility::occlusion::HzbBuilder` is the WGPU-free sizing authority. It converts the effective render size into a half-resolution, power-of-two furthest-depth pyramid:

- `1923x1081` becomes `1024x1024`
- the same case produces `11` mip levels
- reduce work is grouped in batches of up to `4` mips per pass, so the example requires `3` reduce passes
- `1x1` remains valid and produces a single mip

The render graph side now has a built-in `BuiltinRenderFeature::Hzb` descriptor. Default 3D pipelines schedule its `hzb-build` pass after shadow work and before clustered lighting on the ambient-occlusion stage. The pass declares executor `visibility.hzb-build`, reads `scene-depth`, writes the storage texture resource `hzb-furthest`, and carries a `RenderGraphComputeWorkload::hzb_furthest(...)` workload so execution audit can compare the planned dispatch extent with the runtime dispatch record.

`compile.rs` materializes `hzb-furthest` as `Rgba16Float` with the HZB builder dimensions and a full mip chain. `RenderGraphComputeWorkloadDispatchContext` has a dedicated `HzbFurthest` extent, so dispatch auditing uses HZB texture dimensions rather than the full viewport or the clustered-light grid. Runtime graph execution records the HZB dispatch metadata through `record_hzb_build_to_resource(...)`, validates that both depth and HZB resources are bound, and reports storage writes against `hzb-furthest`.

Frame history now reserves a matching HZB history texture beside scene color, GI, AO, and SSR history. The texture is sized from the effective render size, tracks the HZB mip count, imports the previous-frame view as `history.previous.hzb-furthest`, and copies the current `hzb-furthest` mip chain into history at frame end when the compiled pipeline writes HZB. `RenderHistoryCopyReport` exposes `hzb_furthest_copied`, and runtime diagnostics record it at `render.history.copy.hzb_furthest_copied`.

`RenderStats` also exposes HZB-specific progress:

- `last_hzb_mip_count`
- `last_hzb_graph_executed_pass_count`

`update_base_stats(...)` derives the mip count from `HzbBuilder` and counts executed `visibility.hzb-*` executors. Product diagnostics record these as `render.hzb.mip_count` and `render.hzb.graph_executed_pass_count`, and the runtime diagnostics fixture asserts both series.

Current VC-M2 status is intentionally a resource and execution-bookkeeping foundation. The executor validates and records the graph resources and dispatch shape, but the actual WGSL depth-reduce shader is still a follow-up slice. SSR/SSAO still have their existing private pyramid paths until that shader exists and consumers can be migrated safely. HZB algorithm code lives in `graphics/visibility/occlusion`; large existing compile/executor/history files only route resources, stats, and copy behavior.

## Integration Boundaries

`PrimitiveRelevance` lives under `core::framework::render` because it is a renderer-neutral product contract. It does not know about WGPU buffers, mesh pipelines, command replay, or render graph resources.

`parallel_frustum.rs` lives under `graphics::visibility::culling` because it is an implementation detail of the CPU culling pipeline. It consumes `MeshFrustumCandidate` rows built from `VisibilityBounds`, and `is_bounds_visible(...)` is the shared frustum kernel.

This bounds-level kernel is the bridge toward the linear array model described in plan 04: visibility can now evaluate extracted bounds without holding mesh snapshot references, while `VisibilityBvhInstance` and history entries continue to receive the same precomputed bounds for compatibility.

`VisibilityContext` still exposes the pre-existing single-view fields for compatibility. The per-view `FrameVisibility` / `ViewVisibilityContext` shape is now present for the main camera and directional shadow views, and the legacy `visible_entities` / `visible_batches` compatibility fields are derived from the explicit main-view result. Custom render-target cameras are not yet populated because the current `RenderFrameExtract` only carries one concrete camera snapshot; `scene_camera_order_report` has ordering metadata but not the per-camera projection/transform payload needed to cull `CustomTarget` views. That dependency belongs to plan 09 camera descriptor work.

## Validation State

Formatting passed for all touched Rust files.

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` passed after the relevance/frustum integration, bounds-kernel follow-up, main-view `FrameVisibility` integration, shadow-view integration, mesh-pass relevance consumption, main-view planning accessor migration, and visibility RenderStats/diagnostics integration, with the repository's existing warning set.

The same scoped check passed after the VC-M2 HZB builder, graph descriptor/resource, runtime dispatch record, HZB history texture/copy, and HZB diagnostics integration. The command returned the repository's existing warning set.

The VC-M2 touched Rust files passed `rustfmt --edition 2021 --check`. A trailing-whitespace scan over the HZB code/docs/session files returned clean, and `git diff --check` over the same scoped file list exited 0 with only Git's LF-to-CRLF notices.

`cargo test -p zircon_runtime --lib hzb --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` did not run the filtered HZB tests because the shared lib-test target currently fails to compile first in unrelated plugin extension bridge test code: `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs` imports missing `crate::plugin::{BridgeInterfaceSnapshot, BridgeInterfaceStatus, BridgeOwnerTransitionReport}`.

Focused lib-test coverage has not returned a clean result yet. One attempt failed before running the filtered tests because unrelated lib-test sources referenced a missing `RuntimePluginDescriptor::with_target_mode`; the latest attempt timed out after 304 seconds while compiling the shared `zircon_runtime` lib-test target. No render visibility test failure was returned.

Those files and long-running test target compilation are outside the render visibility slice and were not changed here. The new source tests are present and should be rerun when the shared lib-test target is buildable within the local time budget.
