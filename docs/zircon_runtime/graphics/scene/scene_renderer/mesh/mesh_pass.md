---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/layouts/create_material_texture_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_motion_vector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_meshes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/shaders/normal_prepass.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/shadow_map.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_scene_content.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/depth_prepass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/opaque_base.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_motion_vector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_meshes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/shaders/normal_prepass.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/shadow_map.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_scene_content.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs::tests::packed_sort_key_clusters_opaque_by_pipeline_before_tie_breaker
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs::tests::packed_sort_key_keeps_transparent_depth_before_pipeline
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs::tests::packed_sort_key_ignores_transparent_pipeline_variant
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_draw_command_list_sorts_by_phase_then_sort_key
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_draw_command_list_reports_draw_and_instance_sources
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_batch_ref_emits_gpu_scene_instance_command
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_draw_command_list_filters_phase_views_without_resorting
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_build_expected_phase_counts_from_batches
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs::tests::cached_mesh_draw_commands_reuse_matching_static_state
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs::tests::cached_mesh_draw_commands_invalidate_changed_material_revision
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs::tests::cached_mesh_draw_commands_reject_dynamic_transparent_and_indirect_batches
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs::tests::processors_emit_expected_mesh_phases
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs::tests::velocity_processor_requires_motion_vector_history_and_previous_transform
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_reuses_pass_pipeline_shape_id
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_separates_pass_and_pipeline_shape
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::tests::prepared_queue_stats_carry_mesh_pass_command_buffer_counts
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::tests::prepared_queue_stats_carry_gpu_scene_counts
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs::tests::render_gpu_scene_static_scene_second_frame_uploads_zero_bytes
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs::tests::render_gpu_scene_single_moving_entity_uploads_only_its_entry
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_gpu_scene_upload_stats
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_executes_skinned_joint_palette_behind_draw_flag
  - cargo test -p zircon_runtime --lib render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib fallback_mesh_shader --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib shader_declares_gpu_scene_group --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib prepared_queue_stats_carry_gpu_scene_counts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib mesh_batch_ref_emits_gpu_scene_instance_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib shader_is_valid_wgsl --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe fallback_mesh_shader --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe reads_gpu_scene_instance_data --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe shader_is_valid_wgsl --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe shader_declares_gpu_scene_group --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe skinned_joint_palette --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe mesh_batch_ref_emits_gpu_scene_instance_command --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Mesh Pass Command Layer

`mesh_pass` is the first MD-M1 command-layer landing point for the mesh draw command pipeline. It introduces the command vocabulary, per-pass processors, per-frame command buffers, and a command-owned WGPU replay helper for the current built-in mesh passes.

## Command Shape

`MeshDrawCommand` captures the immutable recording decision for one mesh draw:

- `phase` and `sort_key` define where the command belongs in the render queue.
- `pipeline_kind`, `pipeline_key`, and `pipeline_variant_id` identify the resolved pass-specific pipeline path.
- `instance_source` records the GPUScene instance span. Built-in shader transforms are driven through `@builtin(instance_index)`, and commands no longer carry a model-uniform or object-bind fallback.
- `gpu_scene_bind_group` optionally overrides the frame GPUScene bind group for skinned draws that need real current or previous palette buffers.
- `material_textures`, `base_color_texture`, `material`, `standard_material`, and `geometry` are command-owned WGPU handles with stable ids for replay-state comparisons.
- `draw_args` distinguishes direct indexed draws from indexed indirect draws.

The handle ids are still temporary pointer-derived ids in this slice, but the command now carries the actual `wgpu::BindGroup`, `Arc<GpuMeshResource>`, and optional indirect argument buffer needed to replay without returning to the source `MeshDraw`. Test-only constructors create id-only handles so command sorting/counting tests do not need a WGPU device.

## Command List

`MeshDrawCommandList` owns a sorted command vector. Sorting currently uses render phase queue order, then the command `sort_key`, then the pipeline variant id as a deterministic tie-breaker. The list also exposes phase-filtered iteration and `MeshDrawCommandListStats`, which splits direct versus indirect draws and sums GPUScene instance spans so multi-instance draws remain visible in stats.

`MeshPassCommandBuffers` is the side-by-side MD-M1 bridge. It builds depth-prepass, shadow, opaque, alpha-mask, transparent, and velocity command lists from existing `MeshDraw` values through `MeshDraw::mesh_pass_batch_ref(sort_key, source_draw_index)`. Command building now receives a `MeshPipelineVariantResolver`; the production path passes `MeshPipelineCache`, while tests use the pure `MeshPipelineVariantRegistry`. The incoming draw ordinal becomes the command tie-breaker and is packed through `packed_sort_key_u64(...)` when each processor emits a command. Its default `stats()` path reports indirect submission as disabled, while `stats_with_indirect_batches(&RenderCapabilitySummary)` evaluates the current backend's GPU-driven submission gate and folds indirect batch counts across all phase lists.

The packed sort key currently keeps render queue and material queue lanes at zero for mesh commands, because the upstream `MeshDraw` adapter no longer carries the original `GeometryPhaseInput` queue metadata by the time commands are built. Within that transitional constraint, opaque/prepass/velocity commands cluster by pipeline variant and material discriminant before their tie-breaker, while transparent commands keep the depth/tie-breaker lane and ignore pipeline state. Plan 09 still owns the final bit layout and the later camera-depth/queue metadata handoff.

`source_draw_index` remains as an audit/debug bridge, but the built-in WGPU recorders no longer use it for binding or drawing. `partition_mesh_draws.rs` and the old `mesh_draw/render_pass_bindings.rs` draw-binding helper file have been removed from the frame path, and the main mesh pass loops now iterate command slices directly.

## Static Command Cache

MD-M2 adds `CachedMeshDrawCommands` beside the command list. The cache key is `(entity, draw_ordinal, phase)`, where `entity` comes from `RenderMeshSnapshot::node_id` and `draw_ordinal` is assigned while expanding a snapshot into one or more raster draw slices. `MeshDraw` carries this identity plus `RenderMeshStaticState`, and `MeshDraw::mesh_pass_batch_ref(...)` forwards both into `MeshBatchRef` so command construction can decide cache eligibility without reopening scene extract data.

Cache eligibility is intentionally narrow: the batch must be direct, prepared, non-transparent, static mobility, non-skinned GPU-skinning, and the phase must be one of depth prepass, shadow, opaque, or alpha-mask. Transparent commands are excluded because depth ordering is camera-dependent, and velocity commands are excluded because static objects do not belong in the motion-vector pass. The static-state revisions must also be authoritative: `transform_static == true`, `geometry_revision != 0`, and `material_revision != 0`.

`RenderMeshStaticState` is contract-level data under `core::framework::render`, with no WGPU handles. World extraction currently marks static transforms from `Mobility::Static`; the renderer fills geometry/material revision signatures from prepared resource revisions exposed by `ResourceStreamer`. The geometry signature includes the source mesh/model id, prepared asset revision, and LOD selection, so changing LOD selection invalidates the command key even if the underlying GPU mesh resource is still the same.

`SceneRendererCore` owns one `cached_mesh_draw_commands` store. `render_compiled_scene` uses the current `mesh_command_generation` as the cache generation: build draws, patch execution-owned indirect args, build command buffers through `build_mesh_pass_command_buffers_cached(...)`, retain current cache entries, then advance the generation. Cache hits reuse the stored immutable command; misses rebuild only the needed phase command and store it. `PreparedMeshQueueStats` now receives `cached_command_hit_count`, `command_rebuild_count`, and `dynamic_command_count` from the command-buffer build rather than assuming every command was dynamic.

## Indirect Batch Planning

GS-M4 adds the first CPU-side indirect submission planner. `RenderCapabilitySummary` now carries `supports_multi_draw_indirect` and `supports_indirect_first_instance`, and `gpu_driven_submission_supported()` requires those flags plus `supports_indirect_draw`. The wgpu backend maps them from requested device features: `MULTI_DRAW_INDIRECT_COUNT` and `INDIRECT_FIRST_INSTANCE`.

`IndirectDrawBatcher::build(...)` is deliberately pure data. When the gate is closed, it creates no args buffer data and reports every command as a fallback draw. When the gate is open, it converts direct indexed commands into `IndexedIndirectArgs`, groups only adjacent commands with the same phase, pipeline kind, pipeline variant, pipeline key, geometry id, material handles, texture handles, and GPUScene bind group id, and records each run as an `IndirectDrawBatch`. Existing `MeshDrawArgs::IndexedIndirect` commands remain on the fallback path for this slice, so execution-owned indirect buffers are not rebatched yet.

`MeshPassIndirectDrawExecutions::build(...)` is the WGPU execution owner for GS-M4. `render_compiled_scene` builds one optional execution buffer per phase after command-buffer generation, uploads the packed `IndexedIndirectArgs` through `wgpu::util::DeviceExt::create_buffer_init`, and threads those buffers through `RenderPassMeshCommandLists`. Built-in prepass, base, shadow, deferred-gbuffer, and motion-vector recorders now replay `MeshDrawCommandStream` values: eligible batches bind state once from the first command and call `RenderPass::multi_draw_indexed_indirect`, while non-batched direct draws and existing indirect commands keep the old per-command fallback path.

`render_compiled_scene` also passes the frame submission capability summary into `MeshPassCommandBuffers::stats_with_indirect_batches(...)`; `PreparedMeshQueueStats`, `RenderStats.last_indirect_*`, and render diagnostics then report `indirect_batch_count`, `indirect_batched_draw_count`, `indirect_fallback_draw_count`, and `indirect_args_count`.

## Processor Skeleton

`MeshPassProcessor` is the pass conversion seam. `MeshBatchRef` is a lightweight adapter that carries the queue profile, `PipelineKey`, sort key, WGPU handles, draw args, and optional GPUScene instance span needed to build commands. `MeshPassBuildContext` owns variant resolution for processor output, so pass processors pick a `MeshPassPipelineKind` and ask the cache-backed resolver for the stable `MeshPipelineVariantId`. Real `MeshDraw` values can now produce the adapter through `MeshDraw::mesh_pass_batch_ref(sort_key, source_draw_index)`, preserving direct versus indirect draw args, cloning the current mesh, bind group, material, texture, and indirect-buffer handles into the command path, and forwarding any GPUScene span assigned during build.

The first processor set is wired into frame preparation and the built-in WGPU mesh recording path:

- `DepthPrepassProcessor` emits `Prepass` commands for early-z eligible opaque and alpha-mask batches.
- `OpaqueBasePassProcessor` emits `Opaque3d` or `AlphaMask3d` base commands.
- `TransparentPassProcessor` emits `Transparent3d` base commands.
- `ShadowPassProcessor` emits `ShadowDepth` or `ShadowDepthAlphaMask` commands for shadow casters.
- `VelocityPassProcessor` emits `MotionVector` commands only for motion-vector-eligible batches with previous transform data.

`SceneRendererCore::render_compiled_scene` now builds these command buffers immediately after `assign_execution_owned_indirect_args`, so indirect argument patching is visible to the command layer. `RenderPassMeshCommandLists` passes command slices into graph execution. Prepass, base scene, shadow, deferred gbuffer, and object motion-vector recording now iterate `MeshDrawCommand` values and bind/draw through `MeshDrawCommandReplayer`. `PreparedMeshQueueStats` carries the current command counts, cache hit/rebuild/dynamic counts, and replay `state_change_count`/`bind_skip_count`.

## Pipeline Variants

`MeshPipelineCache` owns the production `MeshPipelineVariantRegistry`. The registry keys variants by `MeshPassPipelineKind` plus full `PipelineKey`, allocates monotonic nonzero ids, and reuses the same id when the same pass/pipeline shape appears again. Variant id `0` remains reserved for fixed pass-owned pipelines such as depth prepass and shadow-map pipelines, where the concrete WGPU pipeline is not stored in the mesh pipeline cache.

This removes the earlier temporary `PipelineKey` hash id from `MeshDraw::mesh_pass_batch_ref` and makes variant identity stable for later static command caching. Base and motion-vector passes now resolve concrete WGPU pipelines through cache-backed variant ids (`ensure_pipeline_for_variant(...)` and `ensure_motion_vector_pipeline_for_variant(...)`) instead of reading the command's `PipelineKey` at replay time. Fixed depth and shadow pipelines remain outside the cache registry and keep variant id `0`.

## Replay

`MeshDrawCommandReplayer` owns the per-render-pass state cache for command playback. It tracks the last pipeline state key, bind-group ids for the current physical slots, and geometry id. Pipeline changes clear bind and geometry tracking; repeated bind ids are skipped and counted. The caller still resolves pass-owned resources such as scene group0, forward shadow receiver group1, and the concrete render pipeline from `MeshPipelineCache`, while the replayer handles command-owned GPUScene override, material set, geometry, and draw state.

GS-M2 adds `MeshSceneDataBindHandle` as the scene-data handle. `RenderPassMeshCommandLists` carries the frame's GPUScene bind group, and prepass/base/shadow/deferred/motion-vector recorders call `bind_gpu_scene_if_needed` after the concrete pipeline is selected. The legacy `render_scene` overlay mesh path also threads the same handle into `BaseScenePass`, so both compiled and fallback scene recording paths bind the same scene-data group. The physical mesh slots now match the target ABI: group0 scene, group1 forward shadow receiver where a pass needs it, group2 material set, and group3 GPUScene. The group3 layout reserves primitive/instance/light storage plus current/previous skinned palette uniform bindings; non-skinned draws use fallback palette buffers, and skinned draw commands can carry a per-draw GPUScene bind group that overrides the shared frame handle. `zr_gpu_scene.wgsl` declares the matching group3 shader ABI, is prepended into the built-in forward fallback, normal prepass, shadow-map, and deferred geometry shader sources, and supplies the transform, primitive, and palette helper functions those shaders now call from `@builtin(instance_index)`. The pass-local WGSL files no longer declare `ModelUniform`, `model_data`, `SkinnedJointPaletteUniform`, or `@group(1)` palette bindings, and the replayer no longer has a model-bind path.

`RenderPassMeshCommandLists` carries a per-frame `MeshDrawReplayStatsAccumulator`. Each built-in mesh pass records the replayer stats into this accumulator, and `render_compiled_scene` folds the accumulated `state_change_count` and `bind_skip_count` back into `PreparedMeshQueueStats` before producing `SceneRendererCompiledSceneOutputs`.

For GS-M4, the same command-list context also carries optional per-phase `MeshIndirectDrawExecution` buffers. `MeshDrawCommandReplayer::replay_command_stream(...)` walks the original sorted command slice and the batch table together so fallback commands keep their exact order. For a batch, the first command resolves the pipeline, material/GPUScene bind groups, and geometry buffers; the draw call then uses `batch.first_args * INDEXED_INDIRECT_ARGS_STRIDE_BYTES` and `batch.args_count` against the phase-local indirect args buffer.

## GPUScene Stats Bridge

Plan 03 adds a data-side GPUScene sync beside the existing model-uniform path. `build_mesh_draws` writes real pending mesh draws into `GpuScene` after expansion, keeps each `GpuSceneEntry` by stable instance key, assigns the entry span to the production `MeshDraw`, and returns a `GpuSceneUploadReport`; `render_compiled_scene` folds that report plus `GpuScene::stats()` into `PreparedMeshQueueStats`. `update_base_stats` then forwards the values to `RenderStats.last_gpu_scene_*`, including the explicit `RenderGpuSceneUploadPath::DirectQueueWrite` V1 upload policy.

This bridge now reaches the mesh command vocabulary, WGPU pass binding channel, built-in shader consumption layer, material/custom shader ABI diagnostics, and first indirect-submission planning stats: a draw with a GPUScene entry produces `DrawInstanceSource::GpuSceneInstance`, direct indexed draw args carry the entry's first-instance range, built-in mesh passes bind either the shared GPUScene scene-data group or a command-local palette override through the shared replayer, and the built-in mesh shader sources use `@builtin(instance_index)` plus `zr_gpu_scene.wgsl` helpers to read GPUScene transforms, primitive tint/shadow/motion data, and current/previous skinning palettes. GPUScene commands no longer carry a fallback model bind, the Rust model-uniform cache/build path has been removed from `scene_renderer`, and material textures/samplers plus `material_properties` are replayed as one group2 material set.

GS-M3's first upload refinement keeps full-frame extract submission cheap for static scenes. `PendingMeshDraw` carries the extract transform revision, while `GpuScene::write_primitive` and `GpuScene::write_instances` compare incoming data against their CPU shadows before marking ranges dirty. Replaying the same pending draw set on the next frame therefore produces a zero-byte GPUScene upload report; moving one entry changes only its instance record and uploads exactly one `GpuInstanceData` stride. The upload report path now flows into render-product diagnostics as `render.gpu_scene.upload_path.direct_queue_write`, so later staging-ring or render-graph upload work can switch policy without hiding the active path from frame telemetry.

## Validation State

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` passed on 2026-06-12 after the GS-M4 capability gate, CPU indirect batcher, frame-context capability propagation, indirect stats/diagnostics bridge, and WGPU `multi_draw_indexed_indirect` replay slice; it reports 89 existing warnings. Static scans found no remaining production `ModelUniform`/`model_data` shader resources, group4/group5 mesh pass bindings, `MATERIAL_TEXTURE_BIND_GROUP_SLOT`, `bind_material_textures_if_needed`, or mesh compatibility bind-group references under `scene_renderer`. `rustfmt --edition 2021 --check` passed for the touched Rust files, `git diff --check` returned zero, and a touched-file trailing-whitespace scan returned clean.

Focused `cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 7/7, covering GPUScene layout, allocation, dirty-range merge, bind group layout, static second-frame zero-byte upload, single moving-entry one-stride upload, and explicit direct queue-write upload path. Focused `cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 for diagnostics output. Focused `cargo test -p zircon_runtime --lib mesh_batch_ref_emits_gpu_scene_instance_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 after the Rust-side model-uniform removal. Earlier focused runs on 2026-06-12 covered shader instance-data consumption, `shader_is_valid_wgsl`, `shader_declares_gpu_scene_group`, skinned palette helpers, prepared queue GPUScene stats, and fallback mesh shader validity. Fresh `cargo test -p zircon_runtime --lib render_gpu_scene_indirect_batcher --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` attempts after the GS-M4 CPU batcher slice timed out while compiling the lib-test binary after 120 seconds and 300 seconds; a fresh `cargo test -p zircon_runtime --lib mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` attempt after the WGPU replay slice also timed out after 304 seconds while compiling the lib-test binary. Process hygiene was checked afterward; remaining cargo/rustc processes belonged to other target dirs/sessions. Full mesh lib-test evidence and real-adapter WGPU pipeline/render-product coverage remain milestone testing-stage items.
