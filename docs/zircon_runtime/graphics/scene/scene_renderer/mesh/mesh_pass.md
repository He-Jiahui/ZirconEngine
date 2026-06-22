---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/command_sort_input.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/create_material_texture_bind_group_layout.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/depth_prepass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/opaque_base.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/command_sort_input.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_commands_sort_opaque_by_state_bucket_before_depth
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_commands_sort_transparent_by_depth_before_pipeline_bucket
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_report_static_cache_invalidation_reasons
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs::tests::cached_mesh_draw_commands_reuse_matching_static_state
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs::tests::cached_mesh_draw_commands_invalidate_changed_material_revision
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs::tests::cached_mesh_draw_commands_reject_dynamic_transparent_and_indirect_batches
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs::tests::indirect_compaction_metadata_preserves_source_spans_and_prefixes_output_capacity
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs::tests::indirect_compaction_simulation_rewrites_args_to_visible_instance_remap
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs::tests::indirect_compaction_rejects_visible_instance_capacity_overflow
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs::tests::bindable_storage_buffer_size_keeps_zero_capacity_buffers_bindable
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs::tests::mesh_indirect_compaction_resources_reserve_expected_wgpu_usages
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs::tests::mesh_indirect_compaction_resources_clear_outputs_without_rewriting_metadata
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs::tests::mesh_indirect_draw_execution_builds_compaction_plan_from_uploaded_args
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs::tests::mesh_indirect_args_snapshot_counts_zeroed_instance_args
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs::tests::mesh_indirect_draw_execution_sources_readback_from_indirect_args_buffer
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
  - cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-md3-sort-input-0617
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

`MeshDrawCommandList` owns a sorted command vector. Sorting uses render phase queue order, then the command `sort_key`, then the pipeline variant id as a deterministic tie-breaker. The command `sort_key` is generated by `packed_sort_key_u64(...)` from `MeshCommandSortInput`, the resolved pipeline variant, and the material discriminant. Non-transparent phases group by pipeline/material state before coarse depth, while transparent commands keep depth and stable tie-breaker ahead of state buckets so camera order wins. The list also exposes phase-filtered iteration and `MeshDrawCommandListStats`, which splits direct versus indirect draws and sums GPUScene instance spans so multi-instance draws remain visible in stats.

`MeshPassCommandBuffers` is the side-by-side MD-M1 bridge. It builds depth-prepass, shadow, opaque, alpha-mask, transparent, and velocity command lists from existing `MeshDraw` values through `MeshDraw::mesh_pass_batch_ref(...)`. Command building now receives a `MeshPipelineVariantResolver`; the production path passes `MeshPipelineCache`, while tests use the pure `MeshPipelineVariantRegistry`. `build_mesh_draws` constructs `MeshCommandSortInput` from the frame phase queue's `GeometryPhaseInput` plus material sort offsets, then carries that input through `PendingMeshDraw`, `MeshDraw`, and `MeshBatchRef`; source entity and per-source draw ordinal are only the stable tie-breaker. Its default `stats()` path reports indirect submission as disabled, while `stats_with_indirect_batches(&RenderCapabilitySummary)` evaluates the current backend's GPU-driven submission gate and folds indirect batch counts across all phase lists.

The packed sort key now receives depth, depth bias, render queue, material queue, order-in-layer, UI z-index, and tie-breaker from the same sort profile that ordered the frame phase queue. Fallback paths that do not have a phase queue still derive depth from mesh translation Z and a stable entity tie-breaker, so command creation stays deterministic. Plan 09 still owns the final bit layout and broader camera/renderer ordering policy, but the mesh command path no longer loses the current phase queue metadata before command emission.

`source_draw_index` remains as an audit/debug bridge, but the built-in WGPU recorders no longer use it for binding or drawing. `partition_mesh_draws.rs` and the old `mesh_draw/render_pass_bindings.rs` draw-binding helper file have been removed from the frame path, and the main mesh pass loops now iterate command slices directly.

## Static Command Cache

MD-M2 adds `CachedMeshDrawCommands` beside the command list. The cache key is `(entity, draw_ordinal, phase)`, where `entity` comes from `RenderMeshSnapshot::node_id` and `draw_ordinal` is assigned while expanding a snapshot into one or more raster draw slices. `MeshDraw` carries this identity plus `RenderMeshStaticState`, and `MeshDraw::mesh_pass_batch_ref(...)` forwards both into `MeshBatchRef` so command construction can decide cache eligibility without reopening scene extract data.

Cache eligibility is intentionally narrow: the batch must be direct, prepared, non-transparent, static mobility, non-skinned GPU-skinning, and the phase must be one of depth prepass, shadow, opaque, or alpha-mask. Transparent commands are excluded because depth ordering is camera-dependent, and velocity commands are excluded because static objects do not belong in the motion-vector pass. The static-state revisions must also be authoritative: `transform_static == true`, `geometry_revision != 0`, and `material_revision != 0`.

`RenderMeshStaticState` is contract-level data under `core::framework::render`, with no WGPU handles. World extraction currently marks static transforms from `Mobility::Static`; the renderer fills geometry/material revision signatures from prepared resource revisions exposed by `ResourceStreamer`. The geometry signature includes the source mesh/model id, prepared asset revision, and LOD selection, so changing LOD selection invalidates the command key even if the underlying GPU mesh resource is still the same.

`SceneRendererCore` owns one `cached_mesh_draw_commands` store. `render_compiled_scene` uses the current `mesh_command_generation` as the cache generation: build draws, patch execution-owned indirect args, build command buffers through `build_mesh_pass_command_buffers_cached(...)`, retain current cache entries, then advance the generation. Cache hits reuse the stored immutable command; cold misses rebuild only the needed phase command and store it. Static-state mismatches now return an invalidation reason split by transform, geometry, and material revision, so diagnostics can distinguish first-frame cache population from real stale-command invalidation.

`PreparedMeshQueueStats` receives `cached_command_hit_count`, `command_rebuild_count`, `dynamic_command_count`, `cache_miss_count`, and transform/geometry/material invalidation counters from the command-buffer build rather than assuming every command was dynamic. `RenderStats` mirrors these as `last_mesh_command_*` fields, and runtime diagnostics record them under `render.mesh.queue.command_*` paths beside replay `state_change_count` and `bind_skip_count`.

## Indirect Batch Planning

GS-M4 adds the first CPU-side indirect submission planner. `RenderCapabilitySummary` now carries `supports_multi_draw_indirect` and `supports_indirect_first_instance`, and `gpu_driven_submission_supported()` requires those flags plus `supports_indirect_draw`. The wgpu backend maps them from requested device features: `MULTI_DRAW_INDIRECT_COUNT` and `INDIRECT_FIRST_INSTANCE`.

`IndirectDrawBatcher::build(...)` is deliberately pure data. When the gate is closed, it creates no args buffer data and reports every command as a fallback draw. When the gate is open, it converts direct indexed commands into `IndexedIndirectArgs`, groups only adjacent commands with the same phase, pipeline kind, pipeline variant, pipeline key, geometry id, material handles, texture handles, and GPUScene bind group id, and records each run as an `IndirectDrawBatch`. Existing `MeshDrawArgs::IndexedIndirect` commands remain on the fallback path for this slice, so execution-owned indirect buffers are not rebatched yet.

`MeshPassIndirectDrawExecutions::build(...)` is the WGPU execution owner for GS-M4. `render_compiled_scene` builds one optional execution buffer per phase after command-buffer generation, uploads the packed `IndexedIndirectArgs` through `wgpu::util::DeviceExt::create_buffer_init`, and threads those buffers through `RenderPassMeshCommandLists`. Built-in prepass, base, shadow, deferred-gbuffer, and motion-vector recorders now replay `MeshDrawCommandStream` values: eligible batches bind state once from the first command and call `RenderPass::multi_draw_indexed_indirect`, while non-batched direct draws and existing indirect commands keep the old per-command fallback path.

`scene_renderer::mesh` re-exports `IndexedIndirectArgs` from the mesh module surface so sibling scene-renderer tests and HZB occlusion tests can construct indirect argument rows without importing through the private `build_mesh_draws` subtree. The type remains crate-internal; this only keeps tests and sibling renderer code on the same owner boundary used by `BuiltMeshDraws` and mesh pass execution.

`render_compiled_scene` also passes the frame submission capability summary into `MeshPassCommandBuffers::stats_with_indirect_batches(...)`; `PreparedMeshQueueStats`, `RenderStats.last_indirect_*`, and render diagnostics then report `indirect_batch_count`, `indirect_batched_draw_count`, `indirect_fallback_draw_count`, and `indirect_args_count`.

## Indirect Compaction ABI

VC-M3 now uses a clear + atomic compact path for HZB occlusion instead of replaying the source indirect args after setting fully hidden rows to zero. `indirect_compaction.rs` defines the CPU/WGPU ABI used by the HZB shader and by compact replay.

`IndirectCompactionPlan::try_from_args_and_batch_ranges(...)` derives one `IndirectCompactionBatchMetadata` row per uploaded `IndexedIndirectArgs`. Each row records the source arg index, output arg base, draw-count slot, output `visible_instance_base`, and original `first_instance/count` span. The plan reserves visible-instance remap capacity with a prefix sum over the original instance counts. HZB can therefore append visible source instance indices into a phase-local remap buffer while writing compacted args records to each draw batch's own output range.

The module also defines byte-size constants for the metadata buffer, visible instance index buffer, draw-count buffer, and compacted args buffer, plus `INDIRECT_COMPACTION_UNUSED_INSTANCE_INDEX` for unfilled remap slots. `MeshIndirectDrawExecution` builds the plan from the same CPU args slice used for the WGPU source indirect args buffer, then owns `MeshIndirectCompactionResources` for that exact phase-local stream. The resources initialize the metadata storage buffer from the plan, allocate bindable visible-instance remap storage, allocate a per-batch storage/copy/indirect draw-count buffer, and allocate a compacted indirect args buffer. `MeshIndirectCompactionResources::encode_clear_outputs(...)` clears the visible-instance remap allocation, draw-count allocation, and compacted args allocation before a phase HZB occlusion dispatch; metadata remains immutable per execution.

The group3 GPUScene ABI now includes visible-instance remap storage plus uniform remap params. Normal frame and palette bind groups use direct params and a fallback remap buffer, so unchanged draws still interpret `@builtin(instance_index)` as a GPUScene instance. After HZB compaction, `MeshPassIndirectDrawExecutions::attach_visible_remap_scene_bind_groups(...)` builds phase-local remap bind groups, and compact replay binds that group3 override before issuing `multi_draw_indexed_indirect_count(...)`. Draws with command-local GPUScene bind groups, currently skinned palette overrides, stay on the direct path so compact replay does not replace their per-draw palettes.

HZB diagnostics now read back the same replay buffers used by submission. `MeshIndirectDrawExecution::copy_args_to_readback(...)` copies the source args buffer before compaction is marked ready, and copies the compacted args buffer after compaction is ready. In the compact path it also copies the execution-owned draw-count buffer, so `MeshIndirectArgsSnapshot` can report both remaining submitted instances and the sum of compacted draw counts that `multi_draw_indexed_indirect_count(...)` consumes.

## Processor Skeleton

`MeshPassProcessor` is the pass conversion seam. `MeshBatchRef` is a lightweight adapter that carries the queue profile, `PipelineKey`, `RenderPhaseSortComponents`, WGPU handles, draw args, and optional GPUScene instance span needed to build commands. `MeshPassBuildContext` owns variant resolution for processor output, so pass processors pick a `MeshPassPipelineKind` and ask the cache-backed resolver for the stable `MeshPipelineVariantId`. Real `MeshDraw` values can now produce the adapter through `MeshDraw::mesh_pass_batch_ref(...)`, preserving direct versus indirect draw args, cloning the current mesh, bind group, material, texture, and indirect-buffer handles into the command path, forwarding any GPUScene span assigned during build, and preserving the phase queue sort input captured before pending draws are expanded.

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

For GS-M4, the same command-list context also carries optional per-phase `MeshIndirectDrawExecution` buffers. `MeshDrawCommandReplayer::replay_command_stream(...)` walks the original sorted command slice and the batch table together so fallback commands keep their exact order. Before HZB compaction is marked ready, a batch binds state from the first command and calls `multi_draw_indexed_indirect` against the source args buffer. After HZB compaction is marked ready, replay binds the execution's visible-remap group3 override and calls `multi_draw_indexed_indirect_count` against the compacted args buffer, with the count offset taken from `batch.draw_count_index`. HZB compact replay is currently limited to opaque, alpha-mask, and velocity executions; depth-prepass has already run, shadow views cannot reuse main-camera HZB, and transparent draws cannot be atomically compacted without preserving sort order.

## GPUScene Stats Bridge

Plan 03 adds a data-side GPUScene sync beside the existing model-uniform path. `build_mesh_draws` writes real pending mesh draws into `GpuScene` after expansion, keeps each `GpuSceneEntry` by stable instance key, assigns the entry span to the production `MeshDraw`, and returns a `GpuSceneUploadReport`; `render_compiled_scene` folds that report plus `GpuScene::stats()` into `PreparedMeshQueueStats`. `update_base_stats` then forwards the values to `RenderStats.last_gpu_scene_*`, including the explicit `RenderGpuSceneUploadPath::DirectQueueWrite` V1 upload policy.

This bridge now reaches the mesh command vocabulary, WGPU pass binding channel, built-in shader consumption layer, material/custom shader ABI diagnostics, and first indirect-submission planning stats: a draw with a GPUScene entry produces `DrawInstanceSource::GpuSceneInstance`, direct indexed draw args carry the entry's first-instance range, built-in mesh passes bind either the shared GPUScene scene-data group or a command-local palette override through the shared replayer, and the built-in mesh shader sources use `@builtin(instance_index)` plus `zr_gpu_scene.wgsl` helpers to read GPUScene transforms, primitive tint/shadow/motion data, and current/previous skinning palettes. GPUScene commands no longer carry a fallback model bind, the Rust model-uniform cache/build path has been removed from `scene_renderer`, and material textures/samplers plus `material_properties` are replayed as one group2 material set.

GS-M3's first upload refinement keeps full-frame extract submission cheap for static scenes. `PendingMeshDraw` carries the extract transform revision, while `GpuScene::write_primitive` and `GpuScene::write_instances` compare incoming data against their CPU shadows before marking ranges dirty. Replaying the same pending draw set on the next frame therefore produces a zero-byte GPUScene upload report; moving one entry changes only its instance record and uploads exactly one `GpuInstanceData` stride. The upload report path now flows into render-product diagnostics as `render.gpu_scene.upload_path.direct_queue_write`, so later staging-ring or render-graph upload work can switch policy without hiding the active path from frame telemetry.

## Validation State

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` passed on 2026-06-12 after the GS-M4 capability gate, CPU indirect batcher, frame-context capability propagation, indirect stats/diagnostics bridge, and WGPU `multi_draw_indexed_indirect` replay slice; it reports 89 existing warnings. Static scans found no remaining production `ModelUniform`/`model_data` shader resources, group4/group5 mesh pass bindings, `MATERIAL_TEXTURE_BIND_GROUP_SLOT`, `bind_material_textures_if_needed`, or mesh compatibility bind-group references under `scene_renderer`. `rustfmt --edition 2021 --check` passed for the touched Rust files, `git diff --check` returned zero, and a touched-file trailing-whitespace scan returned clean.

The 2026-06-17 MD-M3 sort-input follow-up added `MeshCommandSortInput` and propagated frame phase queue depth/render queue/material queue/order inputs from `build_mesh_draws` into `MeshBatchRef::command(...)`. `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-md3-sort-input-0617` passed with existing warnings after `rustfmt --edition 2021`, and scoped `git diff --check` was clean except for Git LF-to-CRLF notices. The focused source tests for opaque state-bucket-before-depth and transparent depth-before-bucket semantics were added but not executed in this slice because functional implementation is prioritized and broader testing is deferred.

The 2026-06-17 MD-M2 cache-diagnostics follow-up added observable command cache miss and invalidation reasons. `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-md2-cache-diagnostics-0617` passed with existing warnings after `rustfmt --edition 2021`, and scoped `git diff --check` was clean except for Git LF-to-CRLF notices. Focused tests for cache invalidation diagnostics and product diagnostic rows were added in source but not executed in this implementation-first slice.

During the 2026-06-13 editor UI grouped-selection validation, a filtered runtime lib-test compile hit a private-module error because `hzb_occlusion_culler.rs` tests imported `IndexedIndirectArgs` through `scene_renderer::mesh::build_mesh_draws`. Re-exporting `IndexedIndirectArgs` from `scene_renderer::mesh` and updating the test import unblocked the unrelated UI reducer test compile while preserving the private `build_mesh_draws` subtree.

Focused `cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 7/7, covering GPUScene layout, allocation, dirty-range merge, bind group layout, static second-frame zero-byte upload, single moving-entry one-stride upload, and explicit direct queue-write upload path. Focused `cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 for diagnostics output. Focused `cargo test -p zircon_runtime --lib mesh_batch_ref_emits_gpu_scene_instance_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 after the Rust-side model-uniform removal. Earlier focused runs on 2026-06-12 covered shader instance-data consumption, `shader_is_valid_wgsl`, `shader_declares_gpu_scene_group`, skinned palette helpers, prepared queue GPUScene stats, and fallback mesh shader validity. Fresh `cargo test -p zircon_runtime --lib render_gpu_scene_indirect_batcher --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` attempts after the GS-M4 CPU batcher slice timed out while compiling the lib-test binary after 120 seconds and 300 seconds; a fresh `cargo test -p zircon_runtime --lib mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` attempt after the WGPU replay slice also timed out after 304 seconds while compiling the lib-test binary. Process hygiene was checked afterward; remaining cargo/rustc processes belonged to other target dirs/sessions. Full mesh lib-test evidence and real-adapter WGPU pipeline/render-product coverage remain milestone testing-stage items.

The 2026-06-13 VC-M3 indirect compaction ABI follow-up passed `rustfmt --edition 2021 --check` for `indirect_compaction.rs`, `indirect_draw_execution.rs`, and `mod.rs`. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-coremin --message-format short --color never` passed with 69 existing warnings. `cargo test -p zircon_runtime --lib indirect_compaction --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-coremin --message-format short --color never -- --test-threads=1 --nocapture` timed out after 424 seconds while still compiling the shared lib-test target; the leftover cargo/rustc processes for that target dir were stopped, and no filtered test result was returned.

The 2026-06-13 VC-M3 indirect compaction resource follow-up passed `rustfmt --edition 2021 --check` for `indirect_compaction.rs`, `indirect_compaction_resources.rs`, `indirect_draw_execution.rs`, and `mod.rs`; scoped `git diff --check` and the touched-file trailing-whitespace scan were clean except for Git LF-to-CRLF notices. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never` passed with 68 existing warnings. A focused `cargo test -p zircon_runtime --lib indirect_compaction_resources --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never -- --test-threads=1 --nocapture` attempt failed before running the filtered compaction-resource tests because unrelated UI test code in `zircon_runtime/src/ui/component/state_reducer/keyboard.rs:311` hit `String: Borrow<&str>` trait bound error during shared lib-test compilation. No render/mesh-pass error was returned before that blocker.

The 2026-06-13 VC-M3 compaction clear follow-up added source coverage for `encode_clear_outputs(...)`, verifying that the phase-local visible-instance remap and draw-count outputs are cleared without rewriting metadata. This is the execution-order precursor for the later atomic compact shader and compacted replay path. The touched Rust files passed `rustfmt --edition 2021 --check`, and the same core-min scoped `cargo check` target used by the resource follow-up passed again with 68 existing warnings after the clear/resource-declaration slice.

The 2026-06-13 VC-M3 compact replay follow-up added per-batch draw-count metadata, a compacted args buffer, HZB atomic compaction shader writes, group3 visible-instance remap consumption, and `multi_draw_indexed_indirect_count` replay for HZB-cullable phases. The batcher now records `draw_count_index` and keeps command-local GPUScene/palette draws on the direct path. `rustfmt --edition 2021 --check` passed for the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` passed with 68 existing warnings. A focused `cargo test -p zircon_runtime --lib --no-default-features --features core-min indirect_compaction --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never -- --nocapture` attempt failed before running filtered render tests because unrelated lib-test code defines duplicate `register` methods in `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs` and `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/plugin.rs`.

The 2026-06-13 VC-M3 compact diagnostics follow-up extended `MeshIndirectArgsReadback` to copy compact replay draw-count buffers beside the replay args buffer. `MeshIndirectArgsSnapshot` now exposes `compacted_draw_count()`, and source tests assert that readback uses the replay args buffer plus draw-count buffer. `rustfmt --edition 2021 --check` passed for the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` passed with 68 existing warnings. Focused lib-test execution for `hzb_occlusion_indirect_args_summary_saturates_totals` did not finish within 180 seconds or 600 seconds while compiling the shared lib-test target; the residual cargo/rustc processes were stopped.

The 2026-06-18 VC-M3 focused rerun used the warmed `D:\cargo-targets\zircon-runtime-hzb-culler-contract-0618` lib-test binary after the HZB culler source-contract fix. `indirect_compaction` passed 8/8, covering metadata prefix capacity, per-batch compact simulation, overflow rejection, bindable zero-capacity buffers, resource usage, and clear-without-metadata-rewrite. `mesh_indirect_draw_execution` passed 3/3, covering WGPU indirect args buffer ownership, compaction-plan construction, and replay/readback source selection. `multi_draw_indexed_indirect` passed 1/1 for command replayer multi-draw replay. The scoped `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-culler-contract-0618` also passed with the existing warning set.
