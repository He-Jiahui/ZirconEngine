---
related_code:
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/taa_quality.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/resource_limits.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/asset/assets/material/material_control.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_buffer_bundle/taa_resolve_params_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/taa_resolve_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/full_scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/profiled_scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/execute_taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/temporal_history_store.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/shaders/taa_resolve.wgsl
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_editor/src/tests/editing/state.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/taa_quality.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/resource_limits.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/asset/assets/material/material_control.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_buffer_bundle/taa_resolve_params_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/taa_resolve_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/full_scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/profiled_scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/execute_taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/temporal_history_store.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/shaders/taa_resolve.wgsl
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs
plan_sources:
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - user: 2026-06-14 implement WGPU render pipeline architecture code and update plan progress
tests:
  - zircon_runtime/src/core/framework/render/post_process/stack.rs::tests::taa_resolve_declares_history_velocity_and_final_composite_input
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/temporal_history.rs::unavailable_history_disables_taa_and_restores_scene_color_input
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/temporal_history.rs::unavailable_history_keeps_scene_velocity_for_motion_blur
  - zircon_runtime/src/graphics/tests/pipeline_compile/temporal_and_ops.rs::taa_resolve_compiles_temporal_history_pass_when_taa_stack_is_effective
  - zircon_runtime/src/graphics/tests/pipeline_compile/temporal_and_ops.rs::taa_resolve_pass_and_resources_are_absent_when_taa_is_disabled
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::taa_resolve_executor_requires_graph_resources_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::taa_reactive_mask_clear_executor_requires_graph_resources_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::taa_reactive_mask_mesh_executor_requires_graph_resources_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs::tests::taa_reactive_mask_processor_draws_visible_main_view_batches_by_mask_semantics
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_build_expected_phase_counts_from_batches
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_report_indirect_batch_stats_when_gpu_driven_supported
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_exposes_taa_reactive_mask_entry
  - zircon_runtime/src/asset/tests/assets/material.rs::material_owned_taa_reactive_mask_strength_drives_standard_descriptor_without_shader_override
  - zircon_runtime/src/asset/tests/assets/material.rs::material_owned_taa_reactive_mask_strength_reports_invalid_override
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::optional_postprocess_executors_skip_resource_work_when_effects_are_disabled
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/taa_resolve_pipeline.rs::tests::taa_resolve_shader_parses_and_declares_history_outputs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/taa_resolve_pipeline.rs::tests::taa_resolve_fallback_shader_parses_without_depth_texture_sampling
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs::tests::taa_quality_survives_exact_and_fallback_resolution
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::render_quality_profile_preserves_taa_quality_preset
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_params.rs::tests::taa_resolve_params_map_quality_presets_to_blend_and_rejection
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_params.rs::tests::taa_resolve_params_disable_history_weight_when_history_is_invalid
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/temporal_history_store.rs::tests::temporal_history_state_starts_invalid_and_flips_read_write_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/temporal_history_store.rs::tests::temporal_history_state_invalidation_keeps_slots_but_drops_validity
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary/capability_summary.rs::tests::capability_summary_reports_taa_when_offscreen_postprocess_is_available
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs::tests::offscreen_device_limits_cover_renderer_layout_requirements
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs::tests::offscreen_device_limits_keep_hzb_optional_when_adapter_limit_is_lower
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/post_process.rs::tests::post_process_layout_sampled_texture_count_matches_device_request_limit
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::non_storage_texture_formats_do_not_request_storage_binding
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::storage_texture_formats_request_storage_binding
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs::tests::light_grid_params_cpu_layout_matches_wgsl_uniform_size
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_taa_uses_temporal_resolve_seed_frame_when_requested
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_taa_static_empty_scene_history_stays_stable_after_seed_frame
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_taa_dynamic_occlusion_change_converges_after_history_seed
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_taa_authored_reactive_mask_records_material_writer_path
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_taa_transparent_reactive_mask_records_alpha_writer_path
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_taa_particle_transparent_pass_contributes_before_resolve
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_particle_previous_state_suppresses_velocity_gap_stats
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_taa_reactive_mask_queue_count
  - rustfmt --edition 2021 on TP-M3-S1b/S1c/S1d/S1e/S1f/TP-M4-S1/TP-M4-S2/TP-M4-S3/TP-M4-S3b/TP-M4-S3c/TP-M4-S4d/TP-M4-S4e touched Rust files
  - Naga parse/validate for raw and viewport-depth-fallback taa_resolve.wgsl via already-built naga rlib
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed for TP-M3-S1b/S1c/S1d/S1e/S1f/TP-M4-S1/TP-M4-S2/TP-M4-S3/TP-M4-S3b/TP-M4-S3c production code with 65 existing warnings)
  - source scan `HistoryResolve|history_resolve|HISTORY_PREVIOUS_SCENE_COLOR|HISTORY_CURRENT_SCENE_COLOR|HISTORY_OUTPUT_SCENE_COLOR|history.scene-color|post.history-resolve|FrameHistorySlot::SceneColor` under `zircon_runtime/src` (clear after TP-M4-S1)
  - source scan `HistoryResolve|history_resolve|HISTORY_PREVIOUS_SCENE_COLOR|HISTORY_CURRENT_SCENE_COLOR|HISTORY_OUTPUT_SCENE_COLOR|history.scene-color|post.history-resolve|FrameHistorySlot::SceneColor|with_history_resolve` under `zircon_runtime/src` and `zircon_runtime/tests` (clear after TP-M4-S2)
  - source scan `with_history_resolve|HistoryResolve|history_resolve` under `zircon_runtime/src` and `zircon_editor/src` (clear after the 2026-06-15 editor viewport fixture follow-up)
  - source scan `temporal.taa-reactive-mask-clear|taa.reactive-mask|TAA_REACTIVE_MASK|taa_reactive_mask_tex|load_authored_reactive_mask` under `zircon_runtime/src` (expected TP-M4-S2 hits)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed after TP-M4-S2 with 65 existing warnings)
  - source scan `temporal.taa-reactive-mask-mesh|taa-reactive-mask-mesh|TaaReactiveMask|fs_taa_reactive_mask|ensure_taa_reactive_mask_pipeline` under `zircon_runtime/src` (expected TP-M4-S3 hits)
  - cargo test -p zircon_runtime taa_reactive_mask --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed 4 filtered tests)
  - source scan `taa_reactive_mask_strength|material_properties.data8|fs_taa_reactive_mask|GPU_MATERIAL_UNIFORM_MIN_SIZE: usize = 144` under `zircon_runtime/src` (expected TP-M4-S3b hits)
  - cargo test -p zircon_runtime material_owned_taa_reactive_mask_strength --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed 2 filtered tests)
  - cargo test -p zircon_runtime taa_reactive_mask --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed 6 filtered tests after TP-M4-S3b)
  - source scan `TaaReactiveMaterialMask|fs_taa_reactive_material_mask|taa_reactive_material_mask_mesh_pipelines|create_taa_reactive_material_mask_mesh_pipeline|taa_reactive_mask_strength` under `zircon_runtime/src` (expected TP-M4-S3c hits)
  - cargo test -p zircon_runtime mesh_pass_command_buffers --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed 4 filtered tests after TP-M4-S3c)
  - cargo test -p zircon_runtime taa_resolve_compiles_temporal_history_pass_when_taa_stack_is_effective --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed 1 filtered test)
  - cargo test -p zircon_runtime offscreen_device_limits --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed 2 filtered tests)
  - cargo test -p zircon_runtime post_process_layout_sampled_texture_count_matches_device_request_limit --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (passed 1 filtered test)
  - cargo test -p zircon_runtime texture_formats --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never (passed 2 filtered tests)
  - cargo test -p zircon_runtime render_product_taa_uses_temporal_resolve_seed_frame_when_requested --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never (passed 1 filtered test)
  - cargo test -p zircon_runtime render_product_taa_static_empty_scene_history_stays_stable_after_seed_frame --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never (passed 1 filtered test)
  - cargo test -p zircon_runtime render_product_diagnostics_record_taa_reactive_mask_queue_count --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never (passed 1 filtered test)
  - cargo test -p zircon_runtime render_product_taa_authored_reactive_mask_records_material_writer_path --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never (passed 1 filtered test)
  - cargo test -p zircon_runtime render_product_taa_transparent_reactive_mask_records_alpha_writer_path --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (passed 1 filtered test)
  - cargo test -p zircon_runtime render_product_taa_particle_transparent_pass_contributes_before_resolve --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (passed 1 filtered test)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (passed with 70 existing warnings)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never (passed with 65 existing warnings)
  - cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (blocked by unrelated `virtual_geometry_debug_snapshot_contract.rs` `RenderMeshSnapshot` field drift after the `with_history_resolve` residue was removed)
  - cargo test -p zircon_runtime temporal_history_state --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (timed out after 184s while compiling shared lib-test target; leftover target-dir cargo/rustc stopped)
  - cargo test -p zircon_runtime taa_resolve_shader_parses_and_declares_history_outputs --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (timed out after 364s while compiling shared lib-test target; leftover target-dir cargo/rustc stopped)
  - cargo test -p zircon_runtime taa_resolve --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (blocked before TAA tests by unrelated UI table diagnostics field drift)
  - cargo test -p zircon_runtime taa_quality --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never (blocked before TAA/quality tests by unrelated UI tree-view module and helper drift)
doc_type: module-detail
---

# Temporal TAA Resolve

TP-M3-S1a establishes the graph/resource contract for TAA resolve. TP-M3-S1b adds the first real WGPU execution path: a fullscreen WGSL resolve pass, resource bundle wiring, and scene-color history double buffering inside the current `SceneFrameHistoryTextures` owner. TP-M3-S1c connects TAA quality presets to the quality profile path and upgrades the shader from RGB min/max clamping to a YCoCg variance clip plus depth-delta disocclusion. TP-M3-S1d introduces a dedicated `TemporalHistoryStore` for the TAA scene-color history pair, including key matching, valid/reset state, success-only flip, and store-available seed-frame graph enablement. TP-M3-S1e adds resource-derived responsive/reactive history suppression for high-motion or high-luma-delta pixels. TP-M3-S1f opens the WGPU offscreen TAA product gate and adds seed-frame product test coverage. TP-M4-S1 removes the legacy history resolve feature/effect/executor/resource path and renames the generic frame-history scene-color slot to `TaaSceneColor`. TP-M4-S2 adds the `taa.reactive-mask` graph input, a default clear-zero pass, and shader consumption at TAA resolve binding 5. TP-M4-S3 adds the first authored writer: visible transparent mesh batches write sampled base alpha into the reactive mask before resolve. TP-M4-S3b adds explicit material-authored reactive strength through the standard material path. TP-M4-S3c extends the same mesh pass with CPU-gated opaque/alpha-mask material flag writes. TP-M4-S4a adds a product-level static-history baseline for repeated TAA submissions on the same viewport and tightens WGPU resource limits/usage gates exposed by that product path. TP-M4-S4b adds a dynamic occlusion product baseline and closes the mesh/light-grid/velocity execution gaps that blocked real dynamic TAA frames. TP-M4-S4c exposes authored reactive-mask command counts through product stats/diagnostics and adds a WGPU product test for real material assets with zero and full authored strength. TP-M4-S4d adds the matching WGPU product test for a real transparent Blend material and confirms that its alpha writer contributes one reactive-mask command when the extract phase input is also Blend. TP-M4-S4e adds the particle transparent-pass product baseline, and TP-M1-S9 contracts particle velocity diagnostics when matched previous sprite state is supplied.

This document describes the implemented renderer path. WGPU capability summary now reports `supports_taa` when offscreen post-process rendering is available, so an authored `AntiAliasSettings::taa()` request can enter the TAA seed-frame path. The product tests now cover seed-frame executor selection, a static empty-scene seed/history/history path whose post-seed captured frames stay byte-stable, a dynamic occlusion sequence where repeated occluded frames converge below the visible-to-occluded transition delta, a material-authored opaque reactive-mask writer path where zero strength emits no command and full strength emits one command, a transparent Blend material alpha-writer path that emits one command while the clear/mesh/resolve executors run, and a particle transparent pass that executes before TAA resolve and produces visible RGBA contribution. The particle previous-state product test also proves that motion-blur/TAA frames report missing particle velocity only for sprites without matched previous-state rows. The transparent product baseline explicitly sets `GeometryPhaseInput` to `RenderMaterialAlphaMode::Blend`; a Blend material asset alone changes the mesh pipeline key but does not retroactively repair an Opaque phase queue. The same product coverage also verifies that the backend requests enough sampled texture bindings for the full post-process layout, that scene velocity/reactive-mask transient formats do not ask WGPU for unsupported storage usage, that mesh passes can use disabled light-grid fallback buffers when clustered lighting is off, and that the `LightGridParams` CPU uniform layout matches the 128-byte WGSL binding size. RenderDoc evidence and a real particle `scene-velocity` writer remain later acceptance work.

`PostProcessEffectKind::TaaResolve` is the stack-level effect. When the effective anti-aliasing mode is `Taa` and history is available, `PostProcessStackDescriptor` declares:

- initial external input `TAA_HISTORY_PREVIOUS = "taa.history.previous.scene-color"`;
- raw velocity input `SCENE_VELOCITY = "scene-velocity"`;
- authored reactive input `TAA_REACTIVE_MASK = "taa.reactive-mask"`;
- resolve inputs `SCENE_COLOR`, `SCENE_DEPTH`, `SCENE_VELOCITY`, `TAA_HISTORY_PREVIOUS`, and `TAA_REACTIVE_MASK`;
- outputs `TAA_OUTPUT = "taa.output.scene-color"` and `TAA_HISTORY_CURRENT = "taa.history.current.scene-color"`;
- final composite input `TAA_OUTPUT`, with `FinalComposite` depending on `TaaResolve`.

There is no remaining stack-level legacy scene-color history resolve node or history-stripping clone helper. The descriptor receives `history_available` from the frame contract: when it is false, `TaaResolve` and its history/output/reactive-mask resources are omitted at construction time, `SCENE_COLOR` remains the final composite input, and `SCENE_VELOCITY` is retained only when motion blur, SSR, or another explicit consumer requires it.

`feature_descriptors/temporal.rs` now declares `taa-reactive-mask-clear` with executor id `temporal.taa-reactive-mask-clear`, then `taa-reactive-mask-mesh` with executor id `temporal.taa-reactive-mask-mesh`, then `taa-resolve`. The clear pass writes `TAA_REACTIVE_MASK` as an `R8Unorm` graph texture with clear/store ops. The mesh pass reads `SCENE_DEPTH` and load/stores the same mask so visible transparent mesh can add authored alpha response, and visible opaque/alpha-mask materials with nonzero authored strength can add material response, without erasing the clear value or other future writers. `taa-resolve` reads scene color, depth, scene velocity, previous TAA history, and the reactive mask; it writes current TAA history as an external resource and writes TAA output as a graph texture. `post_process.rs` lists `TAA_OUTPUT` as an optional `post-process` input so the compiler can retain it only for an effective TAA stack.

The pipeline compiler has a TAA-specific compatibility rule: default descriptor validation still sees the pass contract, but no-stack compatibility compilation strips `temporal.taa-resolve` and the `post-process` TAA output input. Effective stack compilation keeps `taa-resolve` only when `TaaResolve` is enabled. It also splits raw scene velocity producers from the reconstructed tile/neighbor motion-vector chain, so TAA-only mode enables `velocity-object`/`velocity-camera` without opening tile-max, tile-max-coarse, or neighbor-max.

## WGPU Execute Path

`request_device(...)` requests `POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE = 21`, matching the actual fragment-visible texture entries in `zircon-post-process-bind-group-layout`. The layout entries are built through a reusable `post_process_entries(...)` helper so the unit test counts the same descriptor list that WGPU receives. This keeps TAA/product tests from depending on adapter defaults that are too low for the full post-process resource set.

Render graph texture materialization filters logical `TextureUsage::STORAGE` through WGPU format support before creating concrete textures. `R8Unorm` reactive mask, `R16Float`, and `Rg16Float` scene velocity/intermediate textures keep render-attachment, sampled, and copy usages, but they do not request `STORAGE_BINDING`. Writable storage usage remains enabled for formats currently used as storage-compatible graph textures: `R32Float`, `Rgba8Unorm`, `Rgba16Float`, and `Rgba32Float`.

`temporal.taa-reactive-mask-clear` is a specialized built-in executor that exits early outside effective TAA mode. When TAA is active, it requires a `RenderPassPostProcessStackContext`, resolves the `TAA_REACTIVE_MASK` graph texture view, and calls `ScenePostProcessResources::execute_taa_reactive_mask_clear(...)`, which opens a clear-only render pass labeled `TaaReactiveMaskClearPass`.

`temporal.taa-reactive-mask-mesh` is a mesh-backed post-process executor. It exits early outside effective TAA mode, then requires mesh draw and mesh pipeline contexts, resolves `TAA_REACTIVE_MASK` plus `SCENE_DEPTH`, and replays the `taa_reactive_mask` command stream. `TaaReactiveMaskPassProcessor` emits `MeshPassPipelineKind::TaaReactiveMask` for visible transparent batches, and `MeshPassPipelineKind::TaaReactiveMaterialMask` for visible opaque/alpha-mask batches only when their CPU-side `taa_reactive_mask_strength` is nonzero. The pass uses the standard material/geometry/GPUScene binding path, a fallback shadow-receiver bind group to satisfy the mesh layout, depth compare `LessEqual` without depth writes, and an `R8Unorm` color target with replace writes.

Material-authored reactive strength lives on `StandardMaterialDescriptor::taa_reactive_mask_strength`. Material assets accept it as a material-owned override with default `0.0` and validation range `0..=1`; it is intentionally excluded from shader custom property overrides. Resource streaming copies it into `MaterialRuntime` and the material capture seed. The standard material uniform is 144 bytes, with `MaterialPropertyUniform.data8.x` carrying the clamped strength. `PendingMeshDraw`, `MeshDraw`, and `MeshBatchRef` also carry the clamped value so the mesh pass can skip default opaque/alpha-mask materials before issuing commands. `fs_taa_reactive_mask` writes `max(sampled_base_alpha, data8.x)` for transparent materials, while `fs_taa_reactive_material_mask` writes the explicit material strength for opaque and alpha-mask materials. The internal `PreparedMeshQueueStats.taa_reactive_mask_command_count` is now mirrored as `RenderStats.last_mesh_taa_reactive_mask_command_count` and product diagnostics records it at `render.mesh.queue.taa_reactive_mask_command_count`, so product submissions can prove that authored opaque materials and transparent alpha writers generated the writer command stream instead of only compiling the static graph.

`temporal.taa-resolve` is now a specialized built-in executor rather than a product post-process validation stub. It exits early when the frame is not in effective TAA mode, otherwise it reads attachment ops for `TAA_OUTPUT` and `TAA_HISTORY_CURRENT`, requires a `RenderPassPostProcessStackContext`, resolves the graph texture views, queries the TAA history valid flag, and calls `ScenePostProcessResources::execute_taa_resolve(...)`.

`ScenePostProcessResources` owns the TAA resolve bind group layout, uniform buffer, and render pipeline alongside the other post-process resources. The bind group layout is:

- binding 0: current scene color, non-filtered float texture;
- binding 1: scene depth, either `texture_depth_2d` or a float fallback texture for GL/ANGLE-style backends;
- binding 2: scene velocity, non-filtered float texture;
- binding 3: previous TAA scene-color history, non-filtered float texture;
- binding 4: `TaaResolveParams` uniform;
- binding 5: TAA reactive mask, non-filtered float texture.

The render pass label is `TaaResolvePass`. It draws a fullscreen triangle and writes two color targets in one pass: `TAA_OUTPUT` as `Rgba16Float`, and `TAA_HISTORY_CURRENT` as `Rgba16Float`. The shader writes the resolved color to both outputs; the history target stores confidence in alpha so next-frame history can ramp in after invalidation.

## Quality Contract

`TaaQualityPreset` is the framework-level quality vocabulary for temporal AA. `AntiAliasSettings` carries it beside the requested AA mode, `AntiAliasFallbackReport` preserves it through TAA fallback reporting, and `RenderQualityProfile::with_taa_quality(...)` lets a profile override the view's authored TAA quality before the effective frame extract is built.

The current renderer keeps quality dynamic through `TaaResolveParams`; it does not add shader defines or pipeline-cache keys. Low, Medium, and High map to two uniform groups:

- history blend weight;
- motion rejection scale;
- YCoCg variance clip gamma;
- depth disocclusion threshold;
- reactive luminance threshold;
- reactive velocity scale;
- full-responsive history multiplier;
- full-responsive confidence cap.

`build_frame_submission_context(...)` applies the quality-profile override before calling `AntiAliasSettings::resolve(...)`. With `supports_taa` now enabled for offscreen-capable WGPU backends, the effective `ViewportRenderFrame` carries the selected TAA quality into `ScenePostProcessResources::execute_taa_resolve(...)`.

For TAA specifically, frame submission distinguishes previous-history validity from history-store availability. If the backend supports TAA and the history store can be allocated, an invalid previous frame no longer forces the graph back to FXAA; the TAA pass still runs and `TaaResolveParams` tells the shader to use zero history weight for that seed frame.

## Shader Resolve

`taa_resolve.wgsl` performs the current temporal resolve:

- loads the current scene color, depth, velocity, and previous history;
- dilates velocity by selecting the closest-depth sample in a 3x3 neighborhood;
- reprojects previous history by subtracting `velocity * viewport_size` from the current pixel center;
- rejects history when the store is invalid, TAA is disabled, reprojection leaves the viewport, or depth is at a clear-plane boundary;
- builds a 3x3 YCoCg variance AABB around the current pixel and clips history toward the AABB center;
- reduces history weight as motion grows or depth delta exceeds the selected threshold;
- derives a resource-local responsive factor from current/history luminance delta and velocity;
- loads the authored `taa.reactive-mask`, clamps it to `[0, 1]`, and takes the maximum with the resource-derived responsive factor before reducing history weight for those pixels;
- stores a confidence value in history alpha so freshly reset history ramps in instead of immediately dominating, and caps responsive-pixel confidence recovery.

This is still not final UE/Bevy parity. The authored reactive-mask texture now has a default zero-valued clear path, a transparent mesh alpha writer V1, explicit material strength, opaque/alpha-mask material flag writing, and product-level command-count evidence for the opaque and transparent material paths: `fallback_mesh.wgsl::fs_taa_reactive_mask` samples base color using the material tint, texture, and vertex color path, reads `MaterialPropertyUniform.data8.x`, discards only when the combined value is <= epsilon, and writes the combined value to the `R8Unorm` mask; `fs_taa_reactive_material_mask` writes clamped material strength for CPU-gated non-transparent materials. The product suite also covers the particle transparent pass in a TAA stack: when particle rendering is enabled, `particle.transparent` executes before `temporal.taa-resolve`; a frame with one particle sprite changes captured RGBA relative to an empty TAA frame, while TAA-only mode keeps `last_particle_velocity_missing_sprite_count` at zero. Particle previous-state velocity writer coverage now exists in the velocity module's product baselines. Exposure/color-space history normalization, explicit camera-cut authoring beyond the current frame-history invalidation key, RenderDoc screenshots, and clean execution of the S20 particle stress-field product route are still pending.

## History Ownership

`SceneFrameHistoryTextures` now owns a `TemporalHistoryStore` for TAA scene color. The store owns two `Rgba16Float` textures created with `TEXTURE_BINDING | COPY_DST | RENDER_ATTACHMENT`, a `TemporalHistoryKey { size, format }`, a read index, and a `valid` bit. Allocation clears both sides to black with zero confidence; resize/format mismatch or missing previous history invalidates the store.

During compiled-scene graph setup, `render_compiled_scene(...)` imports:

- `TAA_HISTORY_PREVIOUS` from `history_textures.taa_scene_color_previous_view()`;
- `TAA_HISTORY_CURRENT` from `history_textures.taa_scene_color_current_view()`.

After successful graph execution, `copy_history_textures(...)` no longer copies the offscreen target scene color into a generic scene-color history texture. TAA calls `TemporalHistoryStore::flip_after_success()`, making the resolved write target become next frame's previous history only after the frame's graph work completed. The first frame after invalidation still runs the resolve pass, but `TaaResolveParams` marks history invalid so shader history weight is zero and the current frame seeds the next history. Non-TAA temporal consumers keep their dedicated ownership paths, such as SSR's own temporal history texture.

This is now the accepted TAA scene-color history ownership path. The current product baselines prove that the same viewport can seed TAA history, keep subsequent static empty-scene captures stable, converge repeated dynamic occlusion frames, execute authored material reactive-mask writer commands, run the transparent alpha writer, and keep a particle transparent pass ahead of TAA resolve while visible sprites contribute to the captured frame. The 2026-06-15 pre-jitter artifact audit found only original plan text, not a recoverable hash artifact, so repository-local acceptance uses the current Off-path product parity baseline. The same date's editor viewport fixture follow-up removed the last known source consumer of the retired `with_history_resolve(...)` helper by using `RenderQualityProfile::with_temporal_history(false)` instead. Remaining Plan 06 work is RenderDoc capture review and broader dynamic-scene visual acceptance.

## Test Coverage

The code now has static contract coverage for stack graph resources, compiler filtering, executor registration, TAA executor failure boundaries, reactive-mask clear/mesh executor failure boundaries, mesh processor selection, shader source parsing, material-owned reactive strength validation, CPU-gated opaque/alpha-mask material command selection, product diagnostics for reactive-mask command counts, depth fallback rewriting, TAA quality preset propagation, responsive uniform/source markers, reactive-mask shader markers, WGPU capability gating, WGPU post-process sampled-texture limit alignment, WGPU transient texture usage filtering, light-grid uniform ABI size, AA pass statistics, product seed-frame selection, product static-history stability, product dynamic-occlusion convergence, product authored/transparent reactive-mask writer command selection, product particle transparent-pass ordering/contribution, store valid/flip state, and legacy history resolve symbol removal. The current executable evidence for TP-M3-S1b/S1c/S1d/S1e/S1f/TP-M4-S1/TP-M4-S2/TP-M4-S3/TP-M4-S3b/TP-M4-S3c/TP-M4-S4a/TP-M4-S4b/TP-M4-S4c/TP-M4-S4d/TP-M4-S4e is:

- raw and fallback `taa_resolve.wgsl` parse and validate through Naga after the YCoCg/depth-delta update;
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed for S1b/S1c/S1d/S1e/S1f/TP-M4-S1/TP-M4-S2/TP-M4-S3/TP-M4-S3b/TP-M4-S3c production code with 65 existing warnings;
- the TP-M4-S2 source scan for the retired feature/effect/executor/resource names, `FrameHistorySlot::SceneColor`, and `with_history_resolve` is clear under `zircon_runtime/src` and `zircon_runtime/tests`;
- the 2026-06-15 editor viewport fixture follow-up scans `zircon_runtime/src` and `zircon_editor/src` for `with_history_resolve|HistoryResolve|history_resolve` and finds no source hits after migrating the fixture to `with_temporal_history(false)`;
- the TP-M4-S2 source scan for `temporal.taa-reactive-mask-clear`, `taa.reactive-mask`, `TAA_REACTIVE_MASK`, `taa_reactive_mask_tex`, and `load_authored_reactive_mask` shows the expected graph/compiler/executor/shader/test hits;
- the TP-M4-S3 source scan for `temporal.taa-reactive-mask-mesh`, `taa-reactive-mask-mesh`, `TaaReactiveMask`, `fs_taa_reactive_mask`, and `ensure_taa_reactive_mask_pipeline` shows the expected graph/compiler/executor/shader/test hits;
- `cargo test -p zircon_runtime taa_reactive_mask --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed 4 filtered tests;
- the TP-M4-S3b source scan for `taa_reactive_mask_strength`, `material_properties.data8`, `fs_taa_reactive_mask`, and `GPU_MATERIAL_UNIFORM_MIN_SIZE: usize = 144` shows the expected material/runtime/uniform/shader/test hits;
- `cargo test -p zircon_runtime material_owned_taa_reactive_mask_strength --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed 2 filtered tests;
- the TP-M4-S3c source scan for `TaaReactiveMaterialMask`, `fs_taa_reactive_material_mask`, `taa_reactive_material_mask_mesh_pipelines`, `create_taa_reactive_material_mask_mesh_pipeline`, and `taa_reactive_mask_strength` shows the expected command/pipeline/cache/shader/dataflow hits;
- `cargo test -p zircon_runtime taa_reactive_mask --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed 6 filtered tests after the material strength and opaque/alpha-mask CPU-gated writer path was added;
- `cargo test -p zircon_runtime mesh_pass_command_buffers --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed 4 filtered tests after the material mask command stats/cache path was added;
- `cargo test -p zircon_runtime taa_resolve_compiles_temporal_history_pass_when_taa_stack_is_effective --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed 1 filtered test;
- `cargo test -p zircon_runtime offscreen_device_limits --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed 2 filtered tests after the post-process sampled-texture limit was moved to `resource_limits.rs`;
- `cargo test -p zircon_runtime post_process_layout_sampled_texture_count_matches_device_request_limit --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614 --message-format short --color never` passed 1 filtered test;
- `cargo test -p zircon_runtime texture_formats --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed 2 filtered tests after the graph transient texture usage mapping was tightened;
- `cargo test -p zircon_runtime render_product_taa_uses_temporal_resolve_seed_frame_when_requested --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed 1 filtered test after the product seed-frame input was aligned with the snapshot-backed render product path;
- `cargo test -p zircon_runtime render_product_taa_static_empty_scene_history_stays_stable_after_seed_frame --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed 1 filtered test;
- `cargo test -p zircon_runtime light_grid_params_cpu_layout_matches_wgsl_uniform_size --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed 1 filtered test after the light-grid uniform was padded to the WGSL 128-byte layout;
- `cargo test -p zircon_runtime render_product_taa_dynamic_occlusion_change_converges_after_history_seed --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed 1 filtered test after the DepthPrepass velocity-object path received mesh pipeline/streamer context and forward mesh passes used disabled light-grid fallback buffers when clustered lighting is off;
- `cargo test -p zircon_runtime render_product_diagnostics_record_taa_reactive_mask_queue_count --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed 1 filtered test after the product diagnostic series was added;
- `cargo test -p zircon_runtime render_product_taa_authored_reactive_mask_records_material_writer_path --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed 1 filtered test after the real material-asset product writer baseline was added;
- `cargo test -p zircon_runtime render_product_taa_transparent_reactive_mask_records_alpha_writer_path --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never` passed 1 filtered test after the real Blend material product writer baseline was aligned with explicit Blend `GeometryPhaseInput`;
- `cargo test -p zircon_runtime render_product_taa_particle_transparent_pass_contributes_before_resolve --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never` passed 1 filtered test after the empty-scene expectation was corrected to reflect that enabled particle profiles execute `particle.transparent` even with zero sprites;
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-m3-s1-0614b --message-format short --color never` passed with 65 existing warnings;
- `cargo check -p zircon_runtime --lib --tests ...` is still blocked outside this slice by `virtual_geometry_debug_snapshot_contract.rs` `RenderMeshSnapshot` field drift after the old history-resolve test fixture residue was removed;
- some older focused TAA lib-test routes remain blocked outside this slice by unrelated UI drift or shared-target compile time: the earlier broad `taa_resolve` route hit the UI table diagnostics `scroll_defaulted` field drift, the `taa_quality` route hit tree-view module/helper drift, `temporal_history_state` timed out after 184 seconds while compiling the shared lib-test target, and the S1e shader-source filter timed out after 364 seconds while compiling the same shared lib-test target.
