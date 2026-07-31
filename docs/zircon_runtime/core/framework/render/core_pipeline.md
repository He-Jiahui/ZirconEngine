---
related_code:
  - dev/bevy/crates/bevy_core_pipeline/src/lib.rs
  - dev/bevy/crates/bevy_core_pipeline/src/schedule.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision_field.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_key_breakdown.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_phase.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/asset/assets/scene/camera.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/tests/runtime_camera_core_pipeline_contract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_preview_sky.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_bloom/execute_bloom.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/transparent/mixed_submission.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/record_overlays.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision_field.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_key_breakdown.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_phase.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/asset/assets/scene/camera.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_preview_sky.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_bloom/execute_bloom.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/transparent/mixed_submission.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/record_overlays.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
plan_sources:
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-05-21 continue M10 schedule visibility acceptance checklist
  - user: 2026-05-21 continue Bevy render schedule and submit pipeline evidence mapping
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs::tests::render_sort_key_camera_order_dominates_queue
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs::tests::render_sort_key_opaque_clusters_pipeline_before_depth
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs::tests::render_sort_key_transparent_depth_back_to_front_ignores_cluster
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs::tests::render_sort_key_2d_sorting_layer_then_order_then_y
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs::tests::render_sort_key_ui_z_index_maps_into_overlay_segment
  - zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs::tests::render_sort_key_fixed_representative_order_snapshot
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_key_breakdown.rs::tests::render_sort_key_breakdown_roundtrip
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_uses_unified_queue_layer_depth_order
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_breakdown_explains_depth_and_queue_order
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_breakdown_reports_first_ordering_difference
  - zircon_runtime/src/core/framework/tests.rs::render_product_pipeline_phase_queue_orders_opaque_mask_and_transparent_for_2d_and_3d
  - zircon_runtime/src/core/framework/tests.rs::render_product_camera_projection_and_core_pipeline_are_independent
  - zircon_runtime/src/core/framework/tests.rs::render_product_orthographic_projection_keeps_orthographic_matrix_in_core3d
  - zircon_runtime/tests/runtime_camera_core_pipeline_contract.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs::render_product_sprite_phase_queue_uses_core2d_phase_order_and_transparent_depth_sort
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs::render_product_sprite_phase_queue_honors_queue_and_order_in_layer
  - zircon_runtime/src/graphics/scene/scene_renderer/transparent/mixed_submission.rs::tests::transparent_submission_order_interleaves_meshes_and_sprites_by_sort_key
  - zircon_runtime/src/graphics/scene/scene_renderer/transparent/mixed_submission.rs::tests::transparent_submission_order_ignores_non_transparent3d_sprites
  - zircon_runtime/src/graphics/scene/scene_renderer/transparent/mixed_submission.rs::tests::transparent_sprite_submission_detection_ignores_mesh_phase_items
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs::tests::mesh_draw_command_replayer_rebinds_after_external_pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs::tests::build_sprite_vertices_routes_transparent3d_to_transparent3d_phase
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs::transparent3d_product_interleaves_mesh_and_sprite_pixels_by_phase_sort_key
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs::transparent3d_product_treats_world_space_ui_sprite_as_transparent_member
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs::default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile/default_pipelines.rs::default_forward_plus_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile/default_pipelines.rs::default_deferred_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs::metadata_context_exposes_attachment_ops_for_written_resource
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::preview_sky_executor_requires_preview_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::overlay_executor_requires_overlay_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::sprite_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::mesh_executor_requires_mesh_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::deferred_gbuffer_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::deferred_lighting_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::plugin_render_feature_descriptors_require_explicit_executor_registration
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::particle_plugin_executor_ids_require_explicit_registration
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs::register_pipeline_asset_rejects_plugin_executor_from_descriptor_only
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs::register_pipeline_asset_accepts_plugin_executor_from_explicit_registration
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs::reload_pipeline_rejects_plugin_executor_from_descriptor_only
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs::reload_pipeline_accepts_plugin_executor_from_explicit_registration
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs::sprite_subpasses_apply_graph_attachment_ops_only_to_outer_draws
  - zircon_runtime/src/graphics/tests/project_render.rs::deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Core Pipeline Contracts

## Purpose

`zircon_runtime::core::framework::render::core_pipeline` owns the neutral camera-selected pipeline and render-phase vocabulary. Bevy's `bevy_core_pipeline` is explicitly camera driven: each camera runs a Core2d or Core3d schedule, and those schedules own the default 2D/3D phase families. Zircon maps that idea into DTOs and phase queues instead of copying Bevy's ECS schedules.

Concrete graph passes, WGPU command encoding, render pass assets, and resource preparation stay under `zircon_runtime::graphics`. The framework module names the product phases that scene extraction, sprite extraction, mesh extraction, and pipeline compilation agree on.

## Product Surface

`CorePipelineKind` selects `Core2d` or `Core3d`. `ViewportCameraSnapshot::core_pipeline_kind()` returns the explicitly authored `core_pipeline`; it does not infer a schedule from `ProjectionMode`. `Core3d` is the serde/default value, so perspective and orthographic 3D cameras retain Forward+/Deferred features. Sprite/2D cameras explicitly select `Core2d`. This follows Bevy's separate Camera2d/Camera3d identity and projection components without creating a second renderer.

`RenderPhase` names the shared phase family: 2D opaque, 2D alpha-mask, 2D transparent, 3D opaque, 3D alpha-mask, 3D transparent, prepass, shadow, deferred, post-process, UI, overlay, and debug.

`RenderPhaseItem` is the neutral queue row. It records the entity, phase, sort key, and whether the phase item came from a mesh or sprite source. The source distinction is important because M6A proved sprites must not be confused with particle billboards, and future Mesh2d rendering must not reuse sprite acceptance accidentally.

`RenderQueueValue` is the Unity-style render queue value authority for phase selection. It defines Background 1000, Geometry 2000, AlphaTest 2450, GeometryLast 2500, Transparent 3000, Overlay 4000, and Max 5000. Alpha mode still supplies the default queue, but authored Unity-range values such as 2900 can override the segment before phase selection; small legacy offsets outside the Unity range are clamped to the material offset window so older per-renderer sort offsets continue to resolve deterministically.

Material projection now preserves the same authority on the standard material snapshot. `StandardMaterialDescriptor.render_queue_value` stores the optional resolved `RenderQueueValue` for explicit material-authored queue overrides, while `resolved_render_queue_value()` keeps old raw descriptors deterministic. Asset validation reports blend materials explicitly placed in the opaque/alpha-test queue range as `RenderQueueAlphaModeConflict`, preventing a semi-transparent blend state from silently entering an opaque phase family.

`RenderPhaseQueue` stores sorted phase items and exposes `items_for_phase(...)` for renderer or diagnostics consumers. `build_mesh_phase_queue(...)` and `build_sprite_phase_queue(...)` now take a resolved `RenderQueueValue` before assigning each item to opaque, alpha-mask, transparent, or overlay phases for the selected core pipeline. Source-level `GeometryPhaseInput` and `SpritePhaseExtractInput` may still carry raw authored `render_queue` and `material_queue` integers, but those are folded once into `RenderQueueValue` before the core pipeline receives sort components.

`RenderPhaseSortKey` is the final `u64` framework sort-key newtype. Its only raw representation is the Plan 09 layout `[camera_order:8][queue:13][domain:33][tie_breaker:10]`. `RenderPhaseSortComponents` carries `camera_order`, resolved `queue`, 2D `sorting_layer`/`order_in_layer`/`y_sort`, depth plus `depth_bias`, `ui_z_index`, and the entity tie-breaker. `RenderPhaseSortKeyBreakdown` and `RenderPhaseSortDecision` now explain the same lanes as camera order, queue, domain, tie key, and final entity tie-breaker instead of the old raw render/material queue fields.

`packed_sort_key_u64(...)` is the single packing entry point for both the framework key and the graphics-facing `MeshDrawCommand` sort key. Camera order dominates every other lane. The queue lane uses `RenderQueueValue.raw()` and remains consistent with phase selection. The 33-bit domain is phase-family specific: opaque 3D, prepass, shadow, deferred, post-process, and debug use pipeline cluster, material cluster, and coarse front-to-back depth; transparent 3D uses millimeter back-to-front depth and only then pipeline cluster; 2D phases use sorting layer, order-in-layer, and y-sort; UI/Overlay use `ui_z_index`. The lower 10 bits keep a compact tie key while `RenderPhaseQueueOrderingKey` still compares the full entity id as the final deterministic queue tie-breaker.

The graphics transparent submission path now consumes that same key across sources. `transparent/mixed_submission.rs` merges `MeshDrawCommand` rows and 3D sprite phase items into one transparent sequence, and `BaseScenePass` records the resulting order inside a single `TransparentMixedScenePass`. This closes the previous gap where Sprite and Mesh could share a framework ordering definition but still draw in separate WGPU passes.

## Graphics Integration

`RenderPipelineAsset::default_core2d()` maps the neutral Core2d phases into concrete stages: `Opaque2d`, `AlphaMask2d`, `Transparent2d`, `PostProcess`, `Ui`, `Overlay`, and `Debug`. It enables the built-in sprite, post-process, UI, and debug overlay features.

`RenderPipelineAsset::default_forward_plus()` and `default_deferred()` map Core3d into concrete 3D render pass orders, including prepass/shadow/deferred or lighting stages before post-process, UI, overlay, and debug.

This two-layer design is intentional: the framework contract says what product phase a renderable belongs to, while `zircon_runtime::graphics` decides which graph pass and executor actually draw it.

## Bevy Render Schedule Evidence

Bevy's render foundation has three separate layers that Zircon must not flatten together. `dev/bevy/crates/bevy_render/src/lib.rs:120-128` states that rendering runs in a `RenderApp` sub-app which exchanges data with the main app between main schedule iterations, and may run between main iterations or in parallel when `PipelinedRenderingPlugin` is enabled. `lib.rs:151-208` names the default render schedule sets: extract command application, prepare assets, prepare meshes, create views, specialize, prepare views, queue, queue meshes, queue sweep, phase sort, prepare resources, prepare bind groups, render, cleanup, and post-cleanup.

Bevy's pipelined mode is explicitly separate from the normal render schedule. `dev/bevy/crates/bevy_render/src/pipelined_rendering.rs:68-105` documents the render thread model: sync and extract happen on the main thread, extract commands are applied on the render thread, the render schedule runs there, `RenderExtractApp` can run before I/O, winit events and the main app schedule run in parallel with rendering, and extraction waits for both sides to finish before starting the next frame. `pipelined_rendering.rs:111-122` inserts `RenderExtractApp` only if `RenderApp` exists, and `pipelined_rendering.rs:124-178` moves the render sub-app over bounded channels to the render thread.

Bevy's core pipeline is then camera-schedule driven inside that render app. `dev/bevy/crates/bevy_core_pipeline/src/schedule.rs:1-11` describes Core2d/Core3d schedules as per-camera sub-schedules. `schedule.rs:29-65` defines `Core3d` with `Prepass`, `MainPass`, `EarlyPostProcess`, and `PostProcess`; `schedule.rs:68-104` defines the same staged shape for `Core2d`. The camera driver at `schedule.rs:111-170` iterates sorted cameras, skips invalid window targets, inserts `CurrentView`, runs each camera's schedule, and records which windows were covered.

## Zircon Submit Schedule State

Zircon currently implements a synchronous submit pipeline, not a Bevy render sub-app. `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs:22-123` owns the frame submission sequence: build the submission context, lock framework state, begin optional debugger capture, prepare runtime plugin sidebands, resolve frame history, build the runtime frame, render through the selected compiled pipeline, collect runtime feedback, record the submission, release old history, and update stats.

Context building and preparation are explicit but not ECS schedule sets. `build_frame_submission_context/build.rs:19-150` resolves viewport state, target size, effective pipeline, enabled render features, effective post-process and anti-alias settings, visibility context, and frame-history validation. `prepare_runtime_submission/prepare.rs:8-31` prepares advanced runtime sidebands for Hybrid GI and Virtual Geometry; this is a narrow runtime preparation phase, not a general Bevy `PrepareAssets` / `PrepareResources` stage.

Concrete command encoding is still owned by the compiled-scene renderer. `render/render.rs` keeps depth-prepass, shadow, and ambient-occlusion in the unconditional early graph family; the depth-prepass stage now receives preview-sky overlay context plus `NormalPrepassPipeline` and mesh draw-list context so `sky.preview-*`, `mesh.depth-prepass`, and `deferred.depth-prepass` execute through the graph instead of private SceneRenderer calls. Forward+ lighting runs before the main forward scene path, while Deferred gbuffer and lighting execute inside `render_scene_passes(...)` after the graph-owned background/depth initialization. The same compiled-scene flow then runs the PostProcess stage through graph executor dispatch, copies history, and runs late UI/overlay/debug graph stages before submitting the command buffer and returning graph execution records. `execute_graph_stage.rs` iterates compiled pass stages for a `RenderPassStage`, validates each pass, inserts stage markers, builds a GPU execution context, dispatches the registered executor, and records pass execution.

Render pass execution contexts now expose graph attachment operations through `RenderPassExecutionContext::attachment_ops_for_write(...)`. This is the execution-side bridge for the render-main-chain M2/M3 attachment metadata: SRP compilation decides whether a texture write should clear or load, RenderGraph preserves that metadata on `RenderGraphPassResourceAccess`, and SceneRenderer executors can query the write target directly before translating the decision into WGPU load/store operations. The helper intentionally returns operations only for write accesses on transient texture or external target resources.

The concrete consumers now include preview sky, depth/normal prepass, `ui.screen-space`, sprite, forward mesh, Deferred gbuffer/lighting, `post.stack`, and `overlay.gizmo` executors. Preview-sky executors read the color/depth write operations, open the WGPU background pass through `ViewportOverlayRenderer::record_preview_sky_with_attachment_ops(...)`, and initialize the background before geometry. The prepass executor then reads `scene-depth` and `gbuffer-normal` write operations and maps them to WGPU depth/color load-store operations before calling `NormalPrepassPipeline`. `post.stack` requires the post-process stack context before dispatching SSAO, clustered lighting, bloom, and final post-process recording from the graph stage, replacing the former fixed `execute_post_process_stack(...)` wrapper. `ui.screen-space` reads the `viewport-output` write operation from the pass context and passes the neutral `RenderGraphAttachmentOps` into `ScreenSpaceUiRenderer::record(...)`; the follow-up `render_plan14_ui_font_attachment_test_surface_suppression_static_passed_cargo_deferred_active_lanes` keeps `ScreenSpaceUiRenderer::last_attachment_ops()` test-only so this attachment-op evidence does not become a production UI renderer API. `overlay.gizmo` requires the graph-bound `viewport-output` and `scene-depth` resources plus prepared overlay buffers before it dispatches `ViewportOverlayRenderer::record_overlays(...)`; compiled-scene submission no longer calls that overlay draw path after graph execution. Sprite executors read the `scene-color` write operation, require `scene-depth`, and dispatch `SpriteRenderer::record(...)` through graph stage execution instead of a private scene-renderer bypass. Forward mesh executors split prepared mesh draws into opaque, alpha-mask, and transparent buckets and dispatch `BaseScenePass` through graph stage execution. Deferred executors route `deferred.gbuffer` and `lighting.deferred` through `DeferredSceneResources`, import `gbuffer-albedo` / `gbuffer-normal` from the offscreen target set, and dispatch post-lighting transparent meshes through the existing `mesh.transparent` executor. The legacy fixed `render_scene(...)` path still calls UI with explicit `Load + Store`, preserving existing behavior while graph-owned preview sky, UI, overlay, sprite, mesh, prepass, Deferred, and post-process paths move off private hard-coded rules.

Pipeline compilation and executor registration form Zircon's closest current analogue to Bevy's queue/render split. `render_pipeline_asset/compile.rs:18-90` validates core-pipeline compatibility, renderer assets, stage-to-phase mapping, feature descriptors, required extract sections, capability requirements, and history bindings. `compile.rs:111-180` builds graph passes, stage mappings, executors, queues, dependencies, and resources. `render_pass_executor_registry.rs` registers concrete built-in executors plus a narrow allow-list of product no-op executors, while descriptor-provided plugin executor ids must arrive through explicit plugin executor registrations. Descriptor-aware registry constructors are crate-internal so external plugin code cannot use the registry as an executor-admission shortcut. Compiled pipeline validation rejects missing executors before activation, and `register_pipeline_asset(...)` / `reload_pipeline(...)` now preserve that rule at runtime-framework entry points so linked feature descriptors cannot silently turn plugin passes into runtime-owned no-ops.

## M10J Completion Boundary

| Bevy render schedule area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Render app / render world | Zircon has a runtime render framework with explicit submit context and renderer state locks. It does not have a separate render world, render sub-app, or main/render app data-exchange boundary. | Keep the current runtime framework unless a real parallel render world is needed, but document the divergence and expose enough stage diagnostics that product users can reason about extract/prepare/queue/render phases. |
| Extract / prepare / queue / render sets | Zircon has frame extract DTOs, runtime preparation, graph compilation, graph executor dispatch, and submit stats, but not Bevy's named `RenderSystems` sets. | Add neutral schedule/stage names to diagnostics and acceptance docs before claiming Bevy-like render-stage parity. Avoid forcing ECS schedule semantics into non-ECS runtime internals. |
| Camera schedule execution | Zircon carries explicit per-camera Core2d/Core3d identity independently from projection and orders cameras, but the concrete renderer is still mostly single active-view submit. | Add true multi-camera schedule execution, per-target coverage tracking, split-screen / render-to-texture routing, and uncovered-surface clearing before claiming camera-driven schedule parity. |
| Pipelined rendering | Zircon submit is synchronous from the caller's perspective. It has scoped profiling markers and RenderDoc markers, but no render thread, `RenderExtractApp`, or overlap telemetry. | Keep pipelined rendering as a separate scheduling milestone; do not conflate current synchronous submit stats with Bevy's frame-overlap model. |
| Graph executors | Zircon validates compiled graph executors and executes stage-declared passes through a registry. UI, overlay, sprite, mesh, prepass, Deferred, and post-process executors now fail on missing execution context instead of silently no-oping; plugin descriptor executor ids also require explicit executor registrations. | Extend executor diagnostics with queue choice, culled pass reason, resource residency, pass timing, and backend queue/capability status so renderer behavior is inspectable without Bevy's render world. |

## M10Q Schedule Visibility Acceptance Checklist

M10.2 is the schedule visibility gate. It does not require Zircon to copy Bevy's render sub-app, render world, or pipelined render thread before the default renderer can continue. It does require the synchronous path to expose enough neutral stages and graph decisions that later 2D, 3D, UI, presentation, post-process, AA, diagnostics, and advanced-render slices can identify where a failure occurred.

| Check | Bevy pressure | Zircon current evidence | Promotion requirement |
| --- | --- | --- | --- |
| Render ownership is named before execution. | `dev/bevy/crates/bevy_render/src/lib.rs:120-128` makes rendering a `RenderApp` sub-app, and `lib.rs:344-423` sets up the render sub-app, schedules, startup, recovery, pipeline cache, and render system. | `submit_frame_extract(...)` is a single runtime-framework entry point with scoped context-build, prepare, render, feedback, record, release, and stats steps (`submit/submit.rs:22-123`). | Keep the divergence explicit: M10.2 can promote schedule visibility without claiming render world or sub-app parity. |
| Stage names are observable. | Bevy names `RenderSystems` sets for extract commands, prepare assets, prepare meshes, create views, specialize, queue, phase sort, prepare, bind groups, render, cleanup, and post-cleanup (`bevy_render/src/lib.rs:151-208`), then chains the base schedule (`lib.rs:286-317`). | Zircon already has source-level phases: build submission context, prepare runtime submission, compile pipeline, execute graph stage, post-process, submit, record, update stats. | Add/maintain diagnostics that use stable neutral names for extract/context-build/prepare/queue-or-phase/graph-compile/render/postprocess/present/cleanup. |
| Camera-driven Core2d/Core3d stays separate from global submit and projection math. | Bevy defines Core3d and Core2d schedules as per-camera sub-schedules (`bevy_core_pipeline/src/schedule.rs:31-104`) and Camera2d/Camera3d as identities separate from projection components. | `ViewportCameraSnapshot::core_pipeline` is projected from scene asset/component data and read by `core_pipeline_kind()`; public contracts prove orthographic Core3d and orthographic Core2d both remain valid while matrix construction stays projection-driven. | Multi-camera execution, per-target coverage, uncovered-surface clearing, split-screen, texture-target scheduling, and editor/runtime multi-view routing remain explicit M10.7/M10.2 follow-ups. |
| Graph executor choices are inspectable. | Bevy's queue/phase-sort/prepare/render sets make phase and backend transitions visible. | Zircon `compile.rs:18-180` validates core-pipeline compatibility, enabled feature descriptors, required extract sections, capability requirements, history bindings, stage mappings, queues, dependencies, graph resources, and attachment ops; `execute_graph_stage.rs:80-180` executes pass stages through registered executors and records executor id, stage, queue, declared queue, dependencies, and resources. `RenderPassExecutionContext::attachment_ops_for_write(...)` lets executors consume clear/load/store decisions without pass-name rules, and `ui.screen-space`, `overlay.gizmo`, `post.stack`, sprite, and forward mesh executors now apply or require graph-owned resources instead of private render-target access. | Add culled-pass reason, resource-residency decision, executor availability, effective queue, and pass timing to diagnostics before calling graph scheduling Bevy-complete. |
| Pipelined rendering remains a separate milestone. | Bevy `PipelinedRenderingPlugin` moves rendering to another thread so frame N rendering can overlap frame N+1 simulation (`pipelined_rendering.rs:68-105`) and only installs `RenderExtractApp` when `RenderApp` exists (`pipelined_rendering.rs:111-122`). | Zircon's current submit path is synchronous and uses profiling scopes plus RenderDoc markers, not render-thread channels or overlap telemetry. | Do not use synchronous submit stats as pipelined rendering proof; future work needs render-thread lifecycle, extraction handoff, overlap timing, and shutdown/drop-thread ownership evidence. |
| Validation is stage-scoped. | Bevy schedule sets let failures be localized by set. | Current tests cover phase queue ordering, explicit camera Core2d/Core3d selection independent from projection, default Core2d/forward/deferred pipeline compile order, and core-pipeline mismatch errors. | M10.2 promotion must run focused core-pipeline/submit tests and `cargo check -p zircon_runtime --lib --locked`, or explicitly remain docs-only. |

This checklist records the source-backed gate and current evidence. It does not claim fresh Cargo validation for this checkout.

## Current Limits

This module is not a Bevy `RenderApp` or render graph scheduler. It does not run sub-app schedules, submit command buffers, clear uncovered swapchains, or allocate per-view targets.

The current extraction path is still mostly single-camera. Camera ordering and target routing are now explicit contracts, but true multi-camera Core2d/Core3d schedule execution, split-screen, render-to-texture scheduling, and editor/runtime multi-view routing remain later milestones.

## Test Coverage

`render_product_pipeline_phase_queue_orders_opaque_mask_and_transparent_for_2d_and_3d` proves mesh alpha modes classify into the expected 2D and 3D phase order.

`render_queue_values_select_phase_before_sort_key_order` proves authored queue values can move opaque and blend items into the queue segment's phase before sort-key comparison, while legacy small offsets keep blend items in the transparent segment.

The `render_queue` module tests prove default alpha-mode queues, Unity segment-to-phase mapping, authored Unity queue overrides, and clamped legacy material offsets.

The material queue tests prove `.zmaterial` render-queue overrides project into `RenderQueueValue`, preserve the legacy raw authored fields for import compatibility, and report blend queue/alpha conflicts through material readiness.

The `render_sort_key_*` tests prove the final Plan 09 `u64` lanes: camera order dominates queue/depth, opaque phases cluster pipeline before coarse depth, transparent phases keep back-to-front depth before cluster, 2D phases sort by layer/order/y, Overlay maps z-index into the domain lane, and a fixed representative sample preserves the intended phase/sort order after deleting the previous framework `i128` path. The breakdown regression proves the diagnostic view reuses the same packed key helpers.

The transparent mixed-submission tests prove 3D sprite phase items and transparent mesh commands are interleaved by the same `u64` key, non-3D sprite items are ignored by the 3D transparent pass, and the mesh replayer invalidates cached state after an inserted Sprite pipeline draw. The Sprite routing regression keeps `RenderPassStage::Transparent3d` mapped to `RenderPhase::Transparent3d`. The WGPU product regressions verify the default Forward+ product path renders a green transparent 3D Sprite and red transparent Mesh through the same `mesh.transparent` ordering path, and that a high-`ui_z_index` world-space UI-like transparent Sprite remains a normal `Transparent3d` member sorted by 3D transparent depth instead of becoming a screen-space UI overlay.

`render_product_camera_projection_and_core_pipeline_are_independent` and `runtime_camera_core_pipeline_contract` prove orthographic cameras can explicitly select either Core3d or Core2d without changing projection math; missing serialized camera identity defaults to Core3d.

The pipeline compile tests prove the default Core2d, forward-plus, and deferred pipeline assets map the neutral phases into concrete render pass stage order and required extract sections.

2026-06-02 render-main-chain validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`. `cargo test -p zircon_runtime --lib --locked graph_execution --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 34 graph-execution tests, including the execution-context attachment-op bridge, preview-sky executor registration and missing-context rejection, the depth-prepass executor, the post-process stack executor rejecting missing post-process context, the screen-space UI executor consuming `viewport-output` attachment ops, the `overlay.gizmo` executor rejecting missing overlay context, and sprite/mesh/Deferred executors rejecting missing renderer context instead of no-oping. `cargo test -p zircon_runtime --lib --locked pipeline_compile --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 43 SRP compile tests. `cargo test -p zircon_runtime --lib --locked render_product_post_process --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 10 post-process product tests, `cargo test -p zircon_runtime --lib --locked render_product_submit --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 11 submit tests, and `cargo test -p zircon_runtime --lib --locked render_framework_stats_report_executed_render_graph_passes --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the graph execution stats regression with `preview-sky` followed by `depth-prepass`. These runs emitted only pre-existing UI/accessibility/text warnings outside this core-pipeline lane.
