---
related_code:
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/morph.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/skinned_palette_buffer_slots.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/joint_palette_storage.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/upload.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/geometry_source_selection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/create_material_texture_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_scene_content.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_meshes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/morph.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/upload.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_scene_content.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_meshes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
plan_sources:
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs::tests::render_gpu_scene_layout_matches_wgsl_offsets
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs::tests::render_gpu_scene_id_allocator_reuses_freed_spans_without_aliasing
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs::tests::render_gpu_scene_id_allocator_coalesces_adjacent_free_spans
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs::tests::render_gpu_scene_update_queue_merges_adjacent_dirty_ranges
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs::render_gpu_scene_static_scene_second_frame_uploads_zero_bytes
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs::render_gpu_scene_single_moving_entity_uploads_only_its_entry
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs::render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_gpu_scene_tests.rs::runtime_15_gpu_scene_tests_are_child_owner
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs::tests::render_gpu_scene_rolls_current_transform_into_previous_after_success
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs::tests::render_gpu_scene_roll_marks_previous_valid_without_dirty_upload_when_unchanged
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs::tests::render_gpu_scene_rolls_current_skinned_palette_after_success
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs::tests::render_gpu_scene_drops_previous_skinned_palette_when_current_is_missing
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs::tests::render_gpu_scene_palette_storage_buffers_swap_and_reuse_per_frame
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs::tests::render_gpu_scene_rolls_current_skinned_gpu_source_after_success
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs::tests::render_gpu_scene_drops_previous_skinned_gpu_source_when_current_is_missing
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs::tests::render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings -- --nocapture --test-threads=1 (2026-06-30 Plan 08 HZB/material-pass staged-cache support repair: passed, 1/1; confirms 9 compute-visible GPUScene storage buffers for the 15-slot HZB limit gate)
  - zircon_runtime/src/graphics/scene/gpu_scene/morph.rs::tests::render_gpu_scene_uploads_morph_storage_buffers
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_storage_buffers_upload.rs::runtime_15_morph_storage_buffers_upload_is_wired
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs::tests::morph_payload_projection_keeps_active_position_deltas_and_weights
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_payload_projection.rs::runtime_15_morph_payload_projection_is_wired
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs::tests::morph_payload_projection_keeps_normal_tangent_and_color_delta_rows
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_validates_morphed_geometry_sources_with_payload_slots
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_payload_slot_indexing.rs::runtime_15_morph_payload_slot_indexing_is_wired
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_geometry_source_selection.rs::runtime_15_morph_geometry_source_selection_is_wired
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs::tests::shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_batch_ref_emits_gpu_scene_instance_command
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::tests::prepared_queue_stats_carry_gpu_scene_counts
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_gpu_scene_upload_stats
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
  - cargo test -p zircon_runtime --lib render_gpu_scene_static_scene_second_frame_uploads_zero_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_gpu_scene_single_moving_entity_uploads_only_its_entry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
doc_type: module-detail
---

# GPUScene Data Module

## Purpose

`graphics::scene::gpu_scene` is the data-plane landing point for plan 03. It defines the CPU-side storage-buffer ABI, stable id allocation, dirty-range collection, and the first WGPU storage-buffer owner that later scene-data bind groups and GPU-driven indirect submission will consume.

This module deliberately lives under `zircon_runtime/src/graphics/scene/` rather than `core::framework::render`: the framework layer owns renderer-neutral extract contracts, while GPUScene owns shader-visible data layout and eventually WGPU buffers.

## Related Files

- `mod.rs` mounts the subsystem and re-exports only the small set of types that later renderer code should consume. The 2026-06-24 Plan 03/F12 cleanup (`render_plan03_gpu_scene_root_facade_suppression_cleanup_static_passed_cargo_timeout_active_compile`) removed the root `dead_code`/`unused_imports` suppression and stopped re-exporting child-owned binding helpers, allocators, dirty queues, capacity constants, layout offset constants, and roll-report types.
- `binding.rs` defines the read-only storage bind group layout for primitive, instance, light, morph delta/weight/payload, and VirtualGeometry buffers; its remap params uniform also carries the active light count for shader iteration.
- `gpu_scene.rs` owns primitive, instance, fallback light, fallback morph delta/weight/payload, and fallback VirtualGeometry storage buffers, shared and per-palette scene-data bind groups, CPU shadow vectors, stable-key entries, and the child test mount. The 2026-06-24 Plan 03 GPUScene tests owner split (`render_plan03_gpu_scene_tests_owner_split_static_passed_cargo_deferred_active_compile_lane`) moved headless upload tests into `gpu_scene/tests.rs`, and `runtime_15_gpu_scene_tests_are_child_owner` prevents those tests from returning to the data-plane owner.
- `gpu_scene/tests.rs` owns the headless WGPU upload coverage for static-scene zero-byte second submit, single moving instance range upload, and light buffer growth/unchanged upload behavior.
- `layout.rs` defines `GpuPrimitiveData`, `GpuInstanceData`, `GpuMorphDelta`, `GpuMorphWeight`, and `GpuMorphPayload` Pod structs with explicit stride and offset constants. `GpuInstanceData.morph_payload_slot` is distinct from VirtualGeometry's primitive `payload_slot`.
- `morph.rs` owns the Plan 08 Morph storage buffers upload surface: `GpuScene::upload_morph_buffers(...)` updates typed payload/delta/weight shadows, recreates live storage buffers when capacity changes, writes the complete payloads through `queue.write_buffer`, and rebuilds the group3 bind group when storage handles change. Status: `render_plan08_morph_payload_slot_indexing_check_passed_wgpu_deferred`; `runtime_15_morph_payload_slot_indexing_is_wired` locks the code/docs/status seam, superseding the earlier storage-only status `render_plan08_morph_storage_buffers_upload_check_passed_wgpu_deferred`.
- `prev_morph_weights.rs` owns the Plan 08 previous morph-weight roll surface: mesh draw sync stages direct mesh source weights by stable instance key, including explicit nonempty all-zero rows for zero-start morph velocity, successful submissions copy that map into the previous-weight map for the next frame, and stale entries are removed when the current frame no longer stages source weights. Status: `render_plan08_direct_morph_weight_velocity_product_wgpu_passed_renderdoc_deferred`, building on `render_plan08_morph_previous_weights_velocity_check_passed_product_deferred`.
- `build_mesh_draws/build/morph_payload_upload.rs` owns the Plan 08 Morph payload projection surface: direct mesh active or previous-active position/normal/tangent/color morph deltas become row-aligned `GpuMorphDelta` / `GpuMorphWeight` rows with one `GpuMorphPayload` header, repeated pending draws share one `PendingMorphPayload`, the helper writes `PendingMeshDraw.morph_payload_slot`, and the build step calls `GpuScene::upload_morph_buffers(...)` before draw sync. The weight buffer stores current weights at `GpuMorphPayload.weight_base` and previous weights at `weight_base + target_count` so Velocity can fetch previous morph state without changing the header ABI. Status: `render_plan08_morph_previous_weights_velocity_check_passed_product_deferred`, building on `render_plan08_morph_payload_slot_indexing_check_passed_wgpu_deferred`; `runtime_15_morph_payload_slot_indexing_is_wired` locks the wider code/docs/status seam. The earlier position-only projection status remains `render_plan08_morph_payload_projection_check_passed_wgpu_deferred`.
- `build_mesh_draws/build/geometry_source_selection.rs` consumes `morph_payload_slot` after upload: payload-backed direct non-skinned morphs use `PendingMeshGeometry::GpuMorphed` and `DynamicGpuMorphedSource`, payload-backed skinned morphs use `DynamicGpuSkinnedMorphedSource`, and CPU-baked fallbacks remain Static/Skinned to avoid double morph. Status: `render_plan08_morph_geometry_source_selection_static_passed_wgpu_deferred`; `runtime_15_morph_geometry_source_selection_is_wired` locks the code/docs/status seam.
- `prev_transform.rs` owns the TP-M1 previous-transform roll surface: successful frame submissions copy each live instance's current transform into the previous-transform slot for the next frame.
- `prev_skinned_palette.rs` owns the Plan 06 TP-M1 skinned current/previous palette roll surface and the per-stable-instance WGPU resource owner. Each live skinned instance allocates exactly two `STORAGE | COPY_DST` palette buffers; draw build writes the non-committed slot, binds the committed slot as previous only when skeleton/morph compatibility passes, and swaps slots only after successful queue submission. The following frame reuses the older slot instead of creating another buffer. CPU palette state is rolled in the same success boundary, and stale instance resources are removed with GPUScene registration.
- `skinned_palette_buffer_slots.rs` isolates the two-slot state machine from WGPU ownership, keeping success-only commit and failed-frame retention directly testable without constructing a device.
- `prev_skinned_source.rs` owns the Plan 06 TP-M1 CPU-morphed current/previous source roll surface: mesh draw sync stages the current morphed-but-unskinned GPU mesh source, and successful frame submissions make it available as the next frame's previous velocity source.
- `id_allocator.rs` defines `GpuSceneIdAllocator`, a first-fit span allocator with deferred free reuse.
- `update_queue.rs` collects primitive and instance dirty ranges and drains them as merged byte ranges for upload.
- `upload.rs` contains the current direct `queue.write_buffer` upload helpers.
- `scene_extract.rs` and `scene/world/render.rs` generate the stable mesh identity and transform revision that GPUScene uses as registration input.
- `SceneRendererCore` owns one `GpuScene`, and both compiled-scene and legacy render paths pass it into mesh draw building.
- `mesh/build_mesh_draws/build.rs` mirrors prepared pending mesh draws into GPUScene storage records before command recording.
- `mesh/build_mesh_draws/create_mesh_draw.rs` no longer builds model uniforms; skinned draws that need real current or previous palettes create command-local GPUScene scene-data bind groups over the same storage buffers.
- `mesh_pipeline_cache/new.rs`, `normal_prepass_pipeline/new.rs`, `shadow_map_renderer.rs`, and `deferred/geometry_pipeline/create.rs` now use the target mesh layout slots: group0 scene, group1 forward shadow receiver where needed, group2 material set, and group3 GPUScene scene data.
- `RenderPassMeshCommandLists` carries the frame's `MeshSceneDataBindHandle`, and prepass/base/shadow/deferred/velocity recorders bind it through the mesh command replayer unless a command-local skinned-palette GPUScene bind group overrides it.
- `mesh/shaders/zr_gpu_scene.wgsl` defines the shared WGSL storage ABI and helper functions; the forward fallback, normal prepass, shadow map, deferred geometry, and morphed geometry include source modules prepend or consume it and now read transform, primitive data, current/previous skinning palettes, morph storage, and VirtualGeometry data from GPUScene bindings.
- `prepared_queue.rs`, `backend_types.rs`, `update_stats/base_stats.rs`, and `render_stats_store/product.rs` carry GPUScene counts, upload bytes, upload range counts, the current upload path, and the first GS-M4 indirect batch planning counters into public `RenderStats` and diagnostics.

## Behavior Model

`GpuPrimitiveData` contains bounds, tint/material-derived values, motion/shadow parameters, flags, and the instance span that belongs to the primitive. Its stride is 80 bytes. `GpuInstanceData` contains current and previous transforms plus primitive and payload references. Its stride is 144 bytes. `GpuMorphDelta` stores one padded morph position delta row, and `GpuMorphWeight` stores one scalar weight row for the Plan 08 group3 binding 7/8 upload owner.

The layout constants are manual ABI constants, not computed aliases. Unit tests compare them against `std::mem::offset_of!` and `size_of` so a Rust field reorder or padding change fails before WGSL is wired in.

`GpuSceneIdAllocator` allocates single ids and contiguous spans from the same free-list structure. Released spans enter `pending_free_spans` first and only become reusable after `commit_pending_frees()`. That frame-boundary step prevents a newly registered primitive or instance span from reusing an id that an in-flight command buffer can still reference.

`GpuSceneUpdateQueue` accepts dirty primitive ids and dirty instance spans. Draining sorts ranges, coalesces overlaps, and merges gaps of at most eight entries. The result carries both element-space ranges and byte-space ranges so the upload layer can choose direct `queue.write_buffer` or staged copy without redoing range math.

`GpuScene` maps a stable instance key to `GpuSceneEntry { primitive_index, first_instance_index, instance_count, last_transform_revision, has_rolled_previous_transform }`. Registration allocates one primitive id and one contiguous instance span, writes default CPU shadow records, marks both ranges dirty, and grows WGPU buffers by powers of two when high-water ids exceed the current capacity.

`flush_updates()` is the current upload entry point. On initial creation or buffer growth it writes the active high-water prefix as a full upload. Otherwise it drains merged primitive and instance ranges and writes only those byte ranges. `GpuSceneUploadReport` explicitly records `GpuSceneUploadPath::DirectQueueWrite`; this is the V1 policy for persistent GPUScene storage buffers while staging-ring or render-graph upload ownership remains a later optimization rather than an implicit missing step. `write_primitive()` and `write_instances()` compare incoming data against the CPU shadow first, so a full-frame extract can be replayed every frame without re-marking stable primitive or instance entries dirty. Free spans are committed only after the flush, preserving the same-frame no-aliasing invariant from the allocator.

Plan 06 TP-M1 now gives GPUScene its own previous-transform roll surface. `roll_prev_transforms_after_success()` runs only after a render command buffer is submitted successfully, copying each live `GpuInstanceData.world_from_local` into `prev_world_from_local` for the next frame. If the matrix actually changes, the instance span remains dirty so the next `flush_updates()` refreshes the GPU buffer even when the following frame's current transform is unchanged. If current and previous are already equal, the entry is still marked as having a valid rolled previous transform but no upload range is produced. `previous_world_from_local(entry)` returns a rolled value only after that validity bit is set, preventing first-frame objects from generating synthetic velocity.

Plan 06 TP-M1 also gives GPUScene a renderer-owned previous skinned-palette state. `GpuSceneSkinnedJointPaletteState` stores a compatibility signature, an optional CPU-morphed `morph_shape_signature`, and the fixed `SkinnedMeshJointPaletteStorage`. Mesh draw sync calls `stage_current_skinned_joint_palette(...)` for each live stable key; successful compiled-scene and legacy submissions call `roll_prev_skinned_palettes_after_success()` after `queue.submit(...)`. Missing current palette state removes stale previous state, so a draw that stops being skinned or no longer fits the 256-joint storage ABI does not keep using an old palette.

Frame integration now feeds the shader-visible path for built-in mesh shaders. `build_mesh_draws` expands real mesh draws first, projects direct mesh morph payloads when active morph weights exist, uploads those morph rows through GPUScene, then registers each pending draw with a stable key derived from source entity plus draw ordinal. `PendingMeshDraw` carries the transform revision from extract, while GPUScene writes primitive data from the same tint, shadow, motion, and transform inputs used by the existing mesh path and one instance record. Unchanged primitive/instance payloads are skipped at the CPU shadow comparison point, so the second submission of the same static scene returns a zero-byte upload report. Morph payload upload bytes are folded into the same `GpuSceneUploadReport` as the VirtualGeometry resident upload sidecar instead of creating a parallel telemetry surface. After upload, mesh-build selection routes payload-backed direct morph sources to Morphed or SkinnedMorphed shader variants while preserving CPU-baked morph fallbacks as Static/Skinned. The builder retains only live frame keys and flushes dirty GPUScene ranges before command recording. It records the resulting `GpuSceneEntry` span on the production `MeshDraw`, and every `MeshBatchRef` now requires that span before it can emit a command. For temporal velocity migration, pending draw sync now uses GPUScene's rolled previous transform as the only unskinned object previous-transform source. Skinned GPU-skinning draws receive a previous palette when the previous/current palette signature plus joint count match. For `CpuMorphed` sources, stable morph weights can reuse the current morphed source as the previous shape, while changed morph weights use the previous source mesh rolled by `prev_skinned_source.rs` and bound through the velocity pass's previous-position vertex slot. The effective previous state is written into primitive flags, motion params, and `MeshDraw::has_previous_velocity_transform`; first-frame objects remain ineligible because their entry has not rolled a previous transform yet. `DrawInstanceSource` only carries GPUScene instance ranges, so built-in draws always use first-instance draw args plus `@builtin(instance_index)`.

`GpuScene` owns a scene-data bind group layout and a shared frame bind group. Bindings 0, 1, and 2 expose primitive, instance, and light buffers as read-only storage to vertex, fragment, and compute stages. Bindings 3 and 4 expose the current and previous 256-joint palettes as vertex-visible read-only storage buffers; command-local bind groups provide the per-entity double-buffer override, while non-skinned draws use the renderer-owned empty storage fallback. Binding 6 is the visible-instance remap params uniform; its `values.x` flag selects direct or remapped instance indices, and `values.y` carries the active light count so shaders can ignore spare light-buffer capacity. Bindings 7 and 8 expose morph delta and morph weight storage, and bindings 9 and 10 expose VirtualGeometry page and cluster storage. `flush_updates()` rewrites both direct and remapped params buffers when the light count changes, so those uniform buffers are allocated with `COPY_DST` in addition to `UNIFORM`. The forward-pipeline resource floor is 14 storage buffers per shader stage after the two palette bindings hard-cut from uniform to storage. If primitive, instance, light, morph, or VirtualGeometry buffers grow, GPUScene recreates the shared bind group after replacing the WGPU buffer so later shader consumption cannot keep a stale buffer handle.

The WGPU pipeline bridge now uses the plan's final physical mesh slots. `SceneRendererCore::new_with_icon_source` creates the empty skinned-palette fallback buffer first, passes it into `GpuScene::new`, then creates mesh pipeline resources so the same GPUScene layout is injected into forward mesh, normal prepass, shadow, and deferred geometry pipeline layouts at group3. Group1 is no longer a mesh compatibility placeholder; forward passes use it for the shadow receiver resources, while passes that do not need group1 install no layout there. During compiled-scene rendering, `render_compiled_scene` clones the active GPUScene bind group into a `MeshSceneDataBindHandle`; `RenderPassMeshCommandLists` forwards that handle to graph execution, and `MeshDrawCommandReplayer` binds a command-local GPUScene bind group when present or falls back to the shared frame handle. The legacy `render_scene` path builds the same handle and passes it through overlay mesh recording so the non-compiled path does not keep a separate mesh-bind behavior.

The shader bridge performs the GS-M2 transform/palette reads, LS-M1 light-buffer reads, Plan 08 morph-storage reads, and VirtualGeometry page/cluster reads for built-in and descriptor-driven geometry sources. `zr_gpu_scene.wgsl` defines `ZrGpuPrimitiveData`, `ZrGpuInstanceData`, `ZrGpuLightData`, group3 storage bindings 0-5 and 7-11, and helper functions such as `zr_world_from_local`, `zr_previous_world_from_local`, primitive tint/shadow/motion accessors, `zr_gpu_scene_light_count`, `zr_gpu_light`, `zr_gpu_light_type`, `zr_skinned_joint_matrix`, `zr_previous_skinned_joint_matrix`, `zr_gpu_scene_morph_delta`, and `zr_gpu_scene_morph_weight`. The current and previous palettes use the same fixed Rust/WGSL `256 * mat4x4<f32> + vec4<u32>` ABI as read-only storage. The forward fallback, normal prepass, shadow map, deferred geometry, deferred lighting, morphed geometry, and VirtualGeometry shader source modules prepend or consume that include where they need GPUScene data. Forward fallback and deferred lighting now shade from group3 light data instead of scene-uniform light fields. `ModelUniform`/`model_data` declarations and pass-local `SkinnedJointPaletteUniform`/`@group(1)` palette declarations are no longer present in the mesh WGSL sources. Rust-side `ModelUniform`, `ModelUniformCache`, model buffers, model bind groups, and the old mesh compatibility bind slot have been removed from `scene_renderer`. The material set now combines material textures/samplers and `material_properties` at group2 binding10, so built-in and custom material shader ABI checks share the same group2/material plus group3/GPUScene contract.

GS-M4 has started at the command-stat planning layer. WGPU device creation now requests `MULTI_DRAW_INDIRECT_COUNT` and `INDIRECT_FIRST_INSTANCE` when available, RHI/backend capability summaries expose those flags, and `RenderCapabilitySummary::gpu_driven_submission_supported()` gates indirect batching. `IndirectDrawBatcher` consumes sorted mesh commands, converts eligible direct indexed commands into CPU `IndexedIndirectArgs`, groups adjacent commands that share the same phase/pipeline/geometry/material/GPUScene bind identity, and reports fallback counts when the gate is closed or a command already owns an indirect args buffer. `render_compiled_scene` passes the frame capability summary through to command-buffer stats so `RenderStats.last_indirect_*` and `render.mesh.queue.indirect_*` diagnostics describe the batch plan for the current backend. The WGPU pass replay still submits via the existing draw paths; actual `multi_draw_indexed_indirect` execution and args-buffer ownership remain the next GS-M4 slice.

## Design and Rationale

Plan 03 follows Unreal's GPUScene shape at a smaller scale: persistent primitive and instance storage buffers with stable indices, but without UE's float4 SOA tiling or GPU-write delegate surface in V1. Zircon uses typed WGSL storage-buffer structs because the current wgpu shader path benefits from simple AoS mirrors and direct Pod upload.

The allocator uses first-fit plus adjacent-span coalescing instead of a grow-only policy. This keeps static-scene churn from permanently increasing buffer high-water marks while still preserving same-frame no-aliasing through deferred free commits.

The dirty queue is intentionally semantic-light. Extract and renderer diff code will decide what is dirty; the queue only batches indices into upload ranges. This keeps transform/material revision policy out of the low-level upload utility.

## Control Flow

The current GS-M1 frame flow is:

1. Extract produces renderer-neutral mesh snapshots, stable keys, and transform revisions.
2. `SceneRendererCore` passes its `GpuScene` owner into mesh draw construction.
3. `build_mesh_draws` registers or updates the GPUScene entry for each real pending draw, using CPU shadow comparison to mark only changed primitive or instance records dirty, and unregisters entries missing from the live frame set.
4. `build_mesh_draws` attaches each entry's first-instance span to the corresponding `MeshDraw`; skinned draws may also attach a command-local GPUScene bind group with real current/previous palette buffers.
5. `flush_updates()` drains dirty ranges and writes primitive/instance storage buffers through direct `queue.write_buffer` calls.
6. `render_compiled_scene` carries the active GPUScene bind group through `RenderPassMeshCommandLists` so built-in mesh passes can bind the scene-data storage group during command replay.
7. Successful compiled-scene and legacy scene submissions call `roll_prev_transforms_after_success()`, `roll_prev_skinned_palettes_after_success()`, `roll_prev_skinned_gpu_sources_after_success()`, and `roll_prev_morph_weights_after_success()` after `queue.submit(...)`, preparing GPUScene previous transforms, previous skinned palettes, previous CPU-morphed source meshes, and previous morph weights for the next frame without mutating the previous values visible to the current frame's velocity pass.
8. `CompiledSceneDraws` returns `GpuSceneUploadReport`, and `RenderStats` receives primitive count, instance count, dirty count, upload bytes, upload path, free-span count, upload range counts, and GPUScene command instance counts.

The current GS-M2 shader flow uses that same data owner and command span to read current transforms, previous transforms, tint, shadow params, motion params, skinning palettes, morph payload headers, morph deltas, current morph weights, and previous morph weights from GPUScene storage/uniform bindings. GPUScene commands no longer carry an object-bind compatibility handle, the model-uniform cache/build path is gone, and material/custom shader validation now expects group2 material bindings plus group3 GPUScene bindings. The remaining validation work is real-adapter pipeline creation and render-product coverage for the final slot layout.

## Edge Cases and Constraints

All spans must have nonzero length. Zero-length frees or dirty marks are ignored for queueing, while zero-length allocations are rejected. Span end calculations use checked arithmetic to expose index-space overflow.

`GPU_SCENE_INVALID_PAYLOAD_SLOT` is `u32::MAX`. It reserves payload/lightmap indirection for later plans without forcing a second payload buffer in this slice.

The dirty-range merge gap is fixed at eight entries. This intentionally trades a small amount of extra upload bandwidth for fewer copy commands when edits are near each other.

## Test Coverage

Current inline unit tests cover:

- primitive, instance, and morph stride/offset ABI parity with the plan table,
- deferred free reuse so same-frame allocations do not alias pending frees,
- adjacent free-span coalescing while preserving high-water capacity,
- dirty-range sorting, duplicate collapse, gap merging, and byte-range output,
- static-scene second-frame zero upload and single-moving-entity one-instance-stride upload on a headless WGPU device,
- successful-submit previous-transform rolling, including the dirty-upload handoff when previous changes and the no-dirty-validity mark when current and previous are already equal,
- successful-submit previous skinned-palette rolling, including stale previous-palette removal when the current frame stops staging a palette,
- successful-submit previous skinned GPU source rolling, including stale previous-source removal when the current frame stops staging a CPU-morphed source,
- successful-submit previous morph-weight rolling, including explicit zero-weight staging for 0 -> nonzero morph velocity and stale previous-weight removal when the current frame stops staging source weights,
- GPUScene bind group layout binding order, read-only storage types, palette uniform slots, morph storage slots, and VirtualGeometry storage slots,
- group3 GPUScene shader ABI declarations in forward fallback, normal prepass, shadow, and deferred geometry shader source,
- GPUScene instance-data shader consumption in forward fallback, normal prepass, shadow, and deferred geometry vertex/fragment paths,
- GPUScene light-buffer shader consumption in forward fallback and deferred lighting paths,
- GPUScene palette helper consumption in forward fallback, normal prepass, shadow, and deferred geometry shader sources, including absence of the old pass-local group1 palette bindings,
- GPUScene morph buffer upload ownership, including typed delta/weight shadows, bind group rebuild on capacity change, same-size reupload without rebuild, and guarded morph helper consumption in morphed geometry includes,
- direct mesh morph payload projection into GPUScene morph storage rows, including active/previous-active weight filtering, position/normal/tangent/color row projection, current and previous weight blocks, payload header creation, per-pending-draw `morph_payload_slot` writeback, shared pending payload dedupe, production Morphed/SkinnedMorphed geometry-source selection for payload-backed sources, previous-only GPU Morphed velocity handoff, and CPU-baked fallback guards that prevent double morph,
- material/custom shader ABI diagnostics that require group2 texture/sampler bindings, group2 binding10 material uniform, and group3 GPUScene bindings,
- Naga WGSL parsing/validation for all four built-in shader sources after `zr_gpu_scene.wgsl` is prepended,
- mesh batch conversion of a GPUScene entry span into a GPUScene instance command,
- prepared queue propagation of GPUScene count, upload range, upload byte, and upload path statistics,
- CPU indirect batcher grouping/fallback behavior and mesh pass aggregation of indirect batch counts under the GPU-driven capability gate,
- WGPU indirect execution source coverage for phase-local `INDIRECT` args buffers and `multi_draw_indexed_indirect` replay,
- render-product diagnostics for GPUScene primitive/instance counts, dirty entry count, uploaded bytes, selected direct/staging upload path, free spans, and upload range counts.

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` passed on 2026-06-12 after the final slot convergence, material set merge, built-in PBR shader update, custom project test WGSL ABI update, material shader layout diagnostic migration, removal of the old material-uniform-only bind group owner, GS-M3 CPU-shadow diff upload, explicit direct-write upload-path diagnostics, GS-M4 CPU indirect batch planning, and GS-M4 WGPU multi-draw replay wiring; it reports 89 existing warnings. Static scans found no remaining production `ModelUniform`/`model_data` shader resources, group4/group5 mesh pass bindings, `MATERIAL_TEXTURE_BIND_GROUP_SLOT`, `bind_material_textures_if_needed`, or mesh compatibility bind-group references under `scene_renderer`.

On 2026-06-14, the Plan 06 TP-M1 GPUScene previous-transform roll slice added source tests for `prev_transform.rs`, wired both successful render submission paths to roll after `queue.submit(...)`, and then hard-cut mesh draw GPUScene sync so the rolled GPUScene previous transform is the only unskinned object previous-transform source. The CPU viewport object-history path was removed from submit state, `ViewportRenderFrame`, stats, and diagnostics. The follow-up skinned-palette slice added `prev_skinned_palette.rs`, current/previous palette maps on `GpuScene`, successful-submit palette rolling, and draw-side guards for signature/joint-count matching. The temporal velocity naming slice then moved the object velocity recorder to `temporal/velocity/execute_velocity_object.rs`, renamed mesh pass/cache/readiness/stat surfaces to velocity terminology, and moved the camera velocity shader under `temporal/velocity/shaders/velocity_camera.wgsl` while leaving tile-max/neighbor-max motion-blur resource names unchanged. The CPU-morphed previous-shape policy slice added a dedicated prepared-queue, `RenderStats`, and product-diagnostics counter for `skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count`, so those sources no longer look like ordinary missing previous-transform draws. The stable morph-shape slice then gave CPU-morphed sources a mesh/weight signature and allows previous-palette velocity when that signature matches the previous frame. The changing-shape follow-up added `prev_skinned_source.rs`, rolls CPU-morphed source meshes after success, and lets the velocity pipeline bind that previous source through a second vertex buffer when morph weights change. The first scoped `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-cpu-history-cut-0614 --message-format short --color never` passed with the repository warning set. The skinned-palette cargo rerun initially exposed visibility errors (`E0364` private constant re-export and `E0603` private mesh module access), both fixed by keeping the max-joint constant private and re-exporting the palette ABI through `scene_renderer`; the current hard-cut name is `SkinnedMeshJointPaletteStorage`. The final scoped `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never` passed with 65 existing warnings and also covered the velocity naming/file-ownership hard cut, CPU-morphed previous-shape diagnostic split, and stable morph-shape velocity reuse. The 2026-06-15 changing-shape follow-up passes the core-min `cargo check` in `D:\cargo-targets\zircon-runtime-temporal-s4d-0614` plus focused filters for `previous_skinned`, velocity previous-position layout, and velocity previous-geometry command binding.

Focused `cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 7/7 on 2026-06-12 after compiling the lib-test binary. This covers the layout, allocator, dirty range, bind group layout, static second-frame zero-byte upload, and one moving entry uploading exactly `GPU_INSTANCE_DATA_STRIDE` bytes, with `GpuSceneUploadPath::DirectQueueWrite` asserted by the two upload tests. Focused `cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 and proves the public diagnostics path records GPUScene counts, bytes, ranges, and direct-write upload policy. The runtime diagnostics aggregation fixture now registers the fake render framework under `crate::graphics::GRAPHICS_MODULE_NAME` so its `GraphicsModule.Manager.RenderFramework` service name satisfies runtime owner validation; the `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` rerun is pending because active plugin-session test files currently stop the `zircon_runtime` lib-test target from compiling. Focused `cargo test -p zircon_runtime --lib mesh_batch_ref_emits_gpu_scene_instance_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 after the Rust-side model-uniform removal. Earlier focused runs on 2026-06-12 covered shader instance-data consumption, `shader_is_valid_wgsl`, `shader_declares_gpu_scene_group`, skinned palette helpers, prepared queue GPUScene stats, and fallback mesh shader validity. Fresh `cargo test -p zircon_runtime --lib render_gpu_scene_indirect_batcher --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` attempts after GS-M4 CPU batch planning timed out while compiling the lib-test binary after 120 seconds and 300 seconds; `cargo test -p zircon_runtime --lib mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` also timed out after 304 seconds after GS-M4 replay wiring. Process hygiene was checked afterward; remaining cargo/rustc processes belonged to other target dirs/sessions. The milestone testing stage still needs completed focused shader/ABI/lib-test runs plus real-adapter WGPU pipeline/render-product coverage for the final group2/group3 layout and GS-M4 multi-draw execution.

## Plan Sources

This module implements the first data-plane slices from `docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md`, the GS-M2 built-in shader consumption/final slot step, the GS-M3 CPU-shadow diff upload plus threshold-selected direct/staging upload policy, the GS-M4 CPU indirect batch planning/telemetry plus WGPU multi-draw replay slice, the Plan 06 TP-M1 previous-transform, previous skinned-palette, and previous CPU-morphed source roll foundation for object velocity, the Plan 08 morph storage buffer upload owner for group3 bindings 7/8, direct mesh morph payload projection, Morph payload slot indexing for binding 11 plus `GpuInstanceData.morph_payload_slot`, payload-backed production Morphed/SkinnedMorphed geometry-source selection while CPU-baked fallbacks remain Static/Skinned, and previous morph-weight rolling for direct plus skinned Velocity previous-position reconstruction. It still prepares for render-graph upload-node refinement, real-adapter multi-draw validation, RenderDoc morph velocity capture review, particle velocity capture review, jitter, TAA, and history convergence.

## Open Issues

Render-graph upload node integration, GPU-generated draw counts, RenderDoc morph velocity capture, broader product miss=0 validation, and real-adapter render-product validation remain later slices. The current frame-path integration reaches mesh commands, binds group3 GPUScene scene data, uses command-local group3 overrides for real current/previous skinned palettes, chooses direct queue writes below 256 KiB and a persistent three-slot staging-copy ring at or above that merged upload size, produces capability-gated CPU indirect batch telemetry, builds phase-local indirect args buffers, replays eligible batches through `multi_draw_indexed_indirect`, owns morph storage fallback/live buffers, uploads direct mesh morph payload headers plus position/normal/tangent/color rows, writes current/previous morph weight blocks, writes per-instance morph payload slots, routes payload-backed direct morph sources to Morphed/SkinnedMorphed shader ids, keeps CPU-baked fallbacks on Static/Skinned to avoid double morph, and owns the unskinned object previous-transform plus guarded skinned previous-palette/source/morph-weight state for temporal velocity. Focused direct and skinned 0.0 -> 1.0 WGPU product readbacks now prove nonzero `scene-velocity` output for morph-weight changes. Plan 06 still needs RenderDoc acceptance for the temporal chain plus broader particle/TAA visual review. The next runtime validation should create the affected WGPU pipelines on the real adapter and confirm the final group1 shadow receiver, group2 material set, group3 GPUScene layout, instance-index/palette/morph helper shader consumption, indirect execution path, and GPUScene previous-transform/previous-palette/previous-source/previous-morph-weight velocity handoff.

## 2026-07-13 Validation-Fixture ABI Convergence

GPUScene bindings 3 and 4 are read-only storage palettes, so WGPU mesh/deferred/prewarm fixtures now allocate palette buffers with `STORAGE` usage and material shader-layout diagnostics require `StorageBuffer`. The prewarm validation group-1 layout is assembled from the same shadow, reflection-probe, lightmap, volumetric, and light-grid entry helpers as the production forward receiver, including lightmap bindings 23/24/28. Fresh current-source validation passed material ABI diagnostics 4/4, deferred custom-pipeline creation 1/1, mesh cache WGPU validation 7/7, and prewarm validation 2/2.
