---
related_code:
  - dev/bevy/crates/bevy_mesh/src/lib.rs
  - dev/bevy/crates/bevy_mesh/src/mesh.rs
  - dev/bevy/crates/bevy_mesh/src/components.rs
  - dev/bevy/crates/bevy_mesh/src/index.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/core/framework/render/mesh/mesh_kind.rs
  - zircon_runtime/src/core/framework/render/mesh/topology.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/core/framework/render/mesh/mesh_kind.rs
  - zircon_runtime/src/core/framework/render/mesh/topology.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
plan_sources:
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_model_metadata_exposes_mesh_bounds_and_vg_presence
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::mesh_executor_requires_mesh_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::depth_prepass_executor_requires_prepass_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::deferred_gbuffer_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::deferred_lighting_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::prepared_queue_stats_require_repeated_direct_prepared_keys_for_batching
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs::compiled_scene_outputs_carry_prepared_mesh_queue_stats
  - zircon_runtime/src/graphics/tests/project_render.rs::deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Mesh Contracts

## Purpose

`zircon_runtime::core::framework::render::mesh` owns the neutral mesh metadata needed before a concrete renderer chooses vertex layouts, buffers, culling, batching, or material pipeline variants. This follows Bevy's separation between `bevy_mesh::Mesh` asset data and renderer-specific mesh preparation.

The module does not own imported model parsing, scene component storage, GPU buffer allocation, skinning, morph targets, or material binding. It is the shared descriptor surface that assets, scene extraction, product profiles, and diagnostics can use without depending on WGPU implementation details.

## Product Surface

`RenderMeshDescriptor` carries topology, bounds, primitive kind, 2D/3D suitability flags, vertex count, index count, primitive count, and whether a Virtual Geometry payload is present.

`RenderMeshTopology` mirrors the topology family used by Bevy and WGPU: triangle list, triangle strip, line list, line strip, and point list. The descriptor intentionally records topology before pipeline selection so Core2d/Core3d phase queueing can stay independent from concrete index-buffer details.

`RenderMeshBounds` stores min, max, center, and radius. `RenderMeshBounds::from_positions(...)` computes a neutral AABB plus bounding radius from positions, while `from_min_max(...)` constructs the same invariant from prepared resource metadata. `transformed(...)` conservatively transforms the local AABB through translation, rotation, and non-uniform scale by projecting all three half-extent axes; consumers therefore retain an exact transformed AABB and a conservative radius without reopening vertex payloads.

`RenderMeshKind` distinguishes planar 2D meshes from spatial 3D meshes. Asset projection currently treats all-`z == 0` position sets as `Planar2d`, marks those as suitable for both 2D and 3D, and marks non-planar meshes as `Spatial3d`.

## Asset Projection

`MeshAsset::render_mesh_descriptor()` projects authored mesh attributes into the neutral descriptor after validating that position data exists. `ModelPrimitiveAsset::render_mesh_descriptor()` projects model primitives and records Virtual Geometry payload presence.

This keeps model import and mesh validation in the asset layer, while the render framework sees only the product metadata needed for phase selection, fallback diagnostics, and future render asset readiness reporting.

## Current Limits

This is not a full Bevy `MeshPlugin`. It does not register mesh assets, mark changed `Mesh3d` entities, prepare GPU mesh slabs, or own `Mesh2d`/`Mesh3d` render components. `Mesh2dComponent` exists as scene data, but materialized Mesh2d draw execution remains future product work.

The descriptor does not yet expose vertex attribute layouts, morph targets, skinning inverse bind poses, tangent generation, or per-attribute upload readiness. Those belong in later mesh asset and renderer preparation milestones.

## Renderer Graph Integration

The M5 render-main-chain cutover now splits prepared `MeshDraw` rows into opaque, alpha-mask, and transparent buckets using `PipelineKey::is_transparent()` and `PipelineKey::is_alpha_mask()`. The forward compiled-scene path keeps preview sky as the fixed clear owner, then dispatches `mesh.opaque`, `mesh.alpha-mask`, and `mesh.transparent` through `execute_graph_stage(...)` at the existing Core3d order points. Each mesh executor requires mesh draw lists, `MeshPipelineCache`, `ResourceStreamer`, `scene-color`, and `scene-depth`; missing mesh context is a hard executor error rather than a silent no-op.

`BaseScenePass` remains the concrete WGPU mesh draw loop, but it now has a graph-owned attachment-op entry point. Opaque mesh execution deliberately loads `scene-color` even when the graph write metadata says clear, because preview sky has already cleared the target and depth for the fixed forward path. Alpha-mask and transparent mesh stages consume the graph write operation directly.

Depth/normal prepass is now graph-owned for Core3d. The built-in mesh and deferred geometry descriptors declare `scene-depth` and `gbuffer-normal` as `depth-prepass` writes, `mesh.depth-prepass` and `deferred.depth-prepass` are real executors, and `RenderPassGpuExecutionContext` requires a `NormalPrepassPipeline` plus mesh draw lists before recording the pass. The executor translates graph attachment ops into WGPU color and depth load/store operations through `NormalPrepassPipeline::record_with_attachment_ops(...)`.

Deferred now uses the same graph executor boundary for its depth/normal prepass, G-buffer, lighting, and post-lighting transparent mesh stages. Preview sky clear remains fixed scene-renderer work, while `deferred.gbuffer` records the albedo G-buffer through `DeferredSceneResources::record_gbuffer_geometry(...)`, `lighting.deferred` records fullscreen lighting through `DeferredSceneResources::execute_lighting(...)`, and deferred transparent meshes dispatch through `mesh.transparent`. The current Deferred resource contract is intentionally narrow and truthful: the graph declares `scene-depth`, `gbuffer-normal`, `gbuffer-albedo`, external `final-color` as the imported preview/background target, and `scene-color` as the lighting output. It does not declare a standalone `gbuffer-material` resource until the shader path actually produces one.

## Renderer Queue Preparation

M6 queue preparation now has an explicit renderer-side profile before draw lists enter graph executors. `PendingMeshDraw` carries the source `Mobility` from `RenderMeshSnapshot`, and final `MeshDraw` rows retain both mobility and geometry source (`Prepared` versus CPU-produced dynamic geometry for skinned or morphed direct draws). `MeshDraw::queue_profile()` derives the render phase from the material pipeline key and records whether the draw uses indirect execution.

`prepare_mesh_queue(...)` summarizes the prepared draw list into early-z, prepared-geometry, dynamic-geometry, indirect, static-batch, dynamic-batch, and GPU-instancing candidate counters. The compiled-scene graph path consumes the profile immediately for behavior: depth/normal prepass now uses only early-z eligible opaque and alpha-mask draws, so transparent meshes are not submitted to the depth prepass in Forward+ or Deferred.

Those counters now also flow through `SceneRendererCompiledSceneOutputs` into the last-frame `SceneRenderer` runtime outputs, and `update_base_stats(...)` copies them into neutral `RenderStats.last_mesh_*` fields. `collect_runtime_diagnostics(...)` mirrors the same values into runtime `DiagnosticStore` paths under `render.mesh.queue.*`, including early-z, indirect, static-batch, dynamic-batch, and GPU-instancing candidate rows. This is intentionally still a queue-preparation diagnostic, not a Bevy-style mesh allocator diagnostic: it reports current-frame draw/profile/batch/instancing candidates, not GPU slab count, byte residency, allocator pressure, or per-asset lifetime.

## Test Coverage

`render_product_assets_model_metadata_exposes_mesh_bounds_and_vg_presence` proves model primitives project topology, planar kind, 2D/3D suitability, counts, bounds, and Virtual Geometry presence.

The sprite/2D tests prove `Mesh2dComponent` is stored as 2D scene data without being misclassified as a sprite product path. That separation is intentional until Mesh2d rendering has its own acceptance slice.

M6 queue-preparation tests in `mesh/prepared_queue.rs` cover early-z eligibility and repeated direct prepared keys for static batching, dynamic batching, and GPU instancing candidates. `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` now also covers the `render.mesh.queue.*` `DiagnosticStore` bridge. The first validation window on 2026-06-02 could only complete formatting and diff checks because concurrent Windows Cargo sessions were already compiling `zircon_runtime`; the focused Cargo filters `prepared_queue_stats`, `phase_ordered_meshes_follow_extract_phase_queue_instead_of_mesh_vector_order`, `prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask`, and runtime diagnostics should be rerun when the shared compile queue is quiet.

2026-06-02 render-main-chain validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`. `cargo test -p zircon_runtime --lib --locked graph_execution --jobs 1 --color never` passed 29 graph-execution tests, including depth-prepass, mesh, Deferred, post-process stack, UI, and overlay executors rejecting missing renderer context instead of no-oping. `cargo test -p zircon_runtime --lib --locked pipeline_compile --jobs 1 --color never` passed 43 SRP compile tests and verifies Deferred no longer declares `gbuffer-material`. `cargo test -p zircon_runtime --lib --locked render_product_post_process --jobs 1 --color never` passed 10 post-process tests, `cargo test -p zircon_runtime --lib --locked render_product_submit --jobs 1 --color never` passed 11 submit tests, `cargo test -p zircon_runtime --lib --locked render_product_ui --jobs 1 --color never` passed 2 UI/overlay submit tests, and `cargo test -p zircon_runtime --lib --locked render_framework_stats_report_executed_render_graph_passes --jobs 1 --color never` passed the graph execution stats regression. These runs emitted only pre-existing UI/accessibility/text warnings outside this mesh lane.
