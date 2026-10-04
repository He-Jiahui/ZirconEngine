---
related_code:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract/frame.rs
  - zircon_runtime/src/core/framework/render/frame_extract/scene_payload.rs
  - zircon_runtime/src/core/framework/render/frame_extract/shared_scene_domain.rs
  - zircon_runtime/src/core/framework/render/frame_extract/geometry.rs
  - zircon_runtime/src/core/framework/render/frame_extract/view.rs
  - zircon_runtime/src/core/framework/render/frame_extract/post_process.rs
  - zircon_runtime/src/core/framework/render/frame_extract/sprite_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract/sprite_phase_input.rs
  - zircon_runtime/src/core/framework/render/frame_extract/particle.rs
  - zircon_runtime/src/core/framework/render/frame_extract/particle_extract_policy.rs
  - zircon_runtime/src/core/framework/render/frame_extract/particle_gpu_frame.rs
  - zircon_runtime/src/core/framework/render/frame_extract/visibility.rs
  - zircon_runtime/src/core/framework/render/frame_extract/visibility_renderable.rs
  - zircon_runtime/src/core/framework/render/frame_extract/lighting.rs
  - zircon_runtime/src/core/framework/render/frame_extract/debug_overlay.rs
  - zircon_runtime/src/core/framework/render/frame_extract/extract_context.rs
  - zircon_runtime/src/core/framework/render/frame_extract/world_snapshot_handle.rs
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_phase_queue_summary.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_phase.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision_field.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_key_breakdown.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue_ordering_key.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue_summary.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/tests/phase_queue_summary.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/phase_ordering.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/final_target_output.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_frame_extract_geometry.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract/frame.rs
  - zircon_runtime/src/core/framework/render/frame_extract/geometry.rs
  - zircon_runtime/src/core/framework/render/frame_extract/view.rs
  - zircon_runtime/src/core/framework/render/frame_extract/post_process.rs
  - zircon_runtime/src/core/framework/render/frame_extract/sprite_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract/sprite_phase_input.rs
  - zircon_runtime/src/core/framework/render/frame_extract/particle.rs
  - zircon_runtime/src/core/framework/render/frame_extract/particle_extract_policy.rs
  - zircon_runtime/src/core/framework/render/frame_extract/particle_gpu_frame.rs
  - zircon_runtime/src/core/framework/render/frame_extract/visibility.rs
  - zircon_runtime/src/core/framework/render/frame_extract/visibility_renderable.rs
  - zircon_runtime/src/core/framework/render/frame_extract/lighting.rs
  - zircon_runtime/src/core/framework/render/frame_extract/debug_overlay.rs
  - zircon_runtime/src/core/framework/render/frame_extract/extract_context.rs
  - zircon_runtime/src/core/framework/render/frame_extract/world_snapshot_handle.rs
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_phase_queue_summary.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/phase_ordering.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_decision_field.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort_key_breakdown.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue_ordering_key.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue_summary.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/tests/phase_queue_summary.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
plan_sources:
  - user: 2026-06-02 implement ZirconEngine WGPU render main-chain closure plan
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
tests:
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_uses_unified_queue_layer_depth_order
  - zircon_runtime/src/core/framework/tests.rs::geometry_phase_inputs_feed_unified_sort_components_into_queue
  - zircon_runtime/src/core/framework/tests.rs::render_phase_queue_order_exposes_submission_phase_precedence
  - zircon_runtime/src/core/framework/tests.rs::render_phase_item_ordering_key_matches_queue_sort_tuple
  - zircon_runtime/src/core/framework/tests/phase_queue_summary.rs::render_phase_queue_summary_reports_phase_counts_and_ordering_bounds
  - zircon_runtime/src/core/framework/tests/phase_queue_summary.rs::geometry_extract_phase_queue_summary_reports_sorted_bounds
  - zircon_runtime/src/core/framework/tests/phase_queue_summary.rs::sprite_extract_phase_queue_summary_reports_core2d_phase_counts
  - zircon_runtime/src/core/framework/tests/phase_queue_summary.rs::render_frame_phase_queue_summary_merges_geometry_and_sprite_counts
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_breakdown_explains_depth_and_queue_order
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_breakdown_reports_first_ordering_difference
  - zircon_runtime/src/core/framework/tests.rs::geometry_extract_builds_static_mesh_batches_by_resource_key
  - zircon_runtime/src/asset/tests/project/example_vampire.rs::vampire_example_scene_extracts_playable_third_person_meshes
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs::render_product_sprite_phase_queue_honors_material_queue_and_ui_z_index
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_carries_scene_camera_order_report_for_scene_camera
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views
  - zircon_runtime/src/scene/tests/render_extract.rs::explicit_camera_render_frame_extract_has_no_scene_camera_order_report
  - zircon_runtime/src/core/framework/render/camera_ordering.rs::tests::render_camera_order_report_carries_descriptor_render_type
  - zircon_runtime/src/graphics/tests/render_framework_bridge/stats.rs::render_framework_stats_report_scene_camera_ordering_metadata
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_surface_offscreen_submit_and_capture_survive_unbind_noop
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_size_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_requires_render_target_usage
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_requires_renderable_render_target_format
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_render_target_metadata_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_ignores_non_texture_targets
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_waits_for_target_descriptor
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_accepts_matching_srgb_format
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_accepts_linear_rgba_target_for_conversion
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_blocks_prepared_format_drift
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_marks_srgb_texture_ready_for_direct_import
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_keeps_linear_texture_on_conversion_writeback_path
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_blocks_unsupported_target_format
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_blocks_prepared_format_drift
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs::tests::build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs::tests::direct_runtime_frame_submit_projects_resolved_output_target
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs::tests::output_target_texture_id_uses_resolved_texture_target_only
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs::tests::output_target_texture_id_ignores_non_texture_targets
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::output_target_graph_import_report_marks_srgb_texture_ready_for_direct_import
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::output_target_graph_import_report_keeps_linear_texture_on_writeback_path
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs::tests::import_frame_targets_rebinds_final_aliases_to_imported_texture_target
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs::tests::compiled_scene_outputs_can_carry_output_target_graph_import_report
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/final_target_output.rs::tests::final_target_output_reports_suppressed_texture_children_only
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_srgb_target_imports_direct_graph_final_target
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_overlay_stack_preserves_base_composite
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_camera_stack_suppressed_target_output
  - cargo test -p zircon_runtime --lib runtime_session_menu --locked --jobs 1 --target-dir D:\cargo-targets\zircon-vampire-menu-0611 --message-format short --color never -- --nocapture --test-threads=1: initially failed 2026-06-11 on `phase_queue_summary.rs` type inference; fixed by explicitly typing the phase-order span vector before rerun
  - zircon_runtime/src/core/framework/render/capture.rs::tests::captured_frame_new_defaults_to_primary_framework_offscreen_source
  - zircon_runtime/src/core/framework/render/capture.rs::tests::texture_capture_report_distinguishes_direct_import_and_conversion_sources
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::camera_target_writeback_report_separates_copy_and_conversion_debug_markers
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_executes_ready_copy_and_conversion_plans
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_report_maps_ready_and_blocked_plans
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::suppressed_output_target_writeback_report_is_texture_only
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_accepts_matching_source_and_destination
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_rejects_source_size_mismatch
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_rejects_destination_size_mismatch
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs::tests::output_target_texture_usages_prepare_render_target_only_without_sampled_binding
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs::tests::output_target_texture_usages_preserve_copy_and_sampled_authoring_flags
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs::renderdoc_debug_marker_registry_covers_capture_timeline
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/graphics/tests/pipeline_compile/feature_descriptors.rs::compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors
  - cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked unified_sort_components --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked render_product_sprite_phase_queue_honors --jobs 1 --message-format short --color never
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs::render_view_apply_target_size_preserves_descriptor_target_and_layers
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs::render_frame_extract_selected_camera_descriptor_replaces_active_selection_only
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs::render_view_select_camera_descriptor_preserves_explicit_anti_alias
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs::render_frame_extract_visibility_input_preserves_layers_above_legacy_mask_width
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_frame_extract_geometry.rs::runtime_15_frame_extract_geometry_is_child_owner
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_flattens_base_then_overlays_for_submit_order
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_extracts_select_each_sequence_descriptor
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_routes_ui_to_last_primary_stack_terminal_only
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_routes_ui_to_last_base_when_no_primary_base_exists
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs::particle_extract_counts_previous_state_by_entity
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs::particle_extract_consumes_duplicate_entity_previous_state_once_per_row
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs::tests::particle_vertices_filter_sprites_by_selected_camera_layers
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs::tests::particle_velocity_vertices_filter_current_sprites_by_selected_camera_layers
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_skips_core_particle_pass_when_particle_sprites_miss_selected_camera_layers
  - cargo test -p zircon_runtime --lib particle_vertices_filter_sprites_by_selected_camera_layers --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan09-particle-typed-layer-0623 --message-format short --color never -- --test-threads=1 --nocapture (blocked before compilation by current Cargo.lock drift)
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/phase_ordering.rs::tests::phase_ordered_meshes_filter_meshes_by_selected_camera_layers
  - cargo test -p zircon_runtime --lib phase_ordered_meshes_filter_meshes_by_selected_camera_layers --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan09-mesh-typed-layer-0623 --message-format short --color never -- --test-threads=1 --nocapture (blocked before compilation by current Cargo.lock drift)
  - cargo test -p zircon_runtime --lib render_frame_extract_visibility_input_preserves_layers_above_legacy_mask_width --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan09-visibility-input-typed-layer-0623 --message-format short --color never -- --test-threads=1 --nocapture (blocked before compilation by current Cargo.lock drift)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan09-visibility-input-typed-layer-0623-check --message-format short --color never (blocked before compilation by current Cargo.lock drift)
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs::explicit_request_camera_uses_volume_mask_for_post_process_volumes
doc_type: module-detail
---

# Render Frame Extract

## Purpose

`RenderFrameExtract` is the neutral frame DTO submitted through `RenderFramework`. Scene producers
seal cacheable scene domains in `RenderFrameScenePayload`; the DTO combines one shared payload
handle with owned timing and view state so graphics code can compile and execute render graph work
without reading editor state or concrete world internals.

## Shared Scene Contract

`RenderFrameScenePayload` owns the generation-qualified world handle plus geometry, animation,
authored lighting/environment/post-process inputs, debug data, sprites, particles, and visibility.
Each large domain is a `RenderSharedSceneDomain<T>`: cloning a frame clones Arc handles, not scene
vectors. If a submission still needs to derive one domain during the Runtime07 M2 transition,
copy-on-write is isolated to that domain and unchanged domains retain pointer identity.

`RenderFrameExtract` itself owns only `Arc<RenderFrameScenePayload>`, `RenderFrameTiming`, and
`RenderViewExtract`. Editor-camera and timing changes are submission-local and do not mutate the
cache-owned scene generation. World, level, snapshot, and synthetic producers use
`RenderFrameScenePayload::new(...)` plus `RenderFrameExtract::new(...)`; direct struct literals are
not a second construction authority. Renderer-derived environment hydration, particle history,
effective post-process, budget, and view-family state are not authored scene facts and remain
scheduled for migration into the renderer submission context rather than being normalized into the
shared payload contract.

## View Size Contract

`RenderViewExtract` records an optional `target_size` alongside the transitional `camera` snapshot and the descriptor list in `cameras`. During the Plan 09 camera hard cutover, `CameraRenderDescriptor` owns the selected camera target, viewport rectangle, render order, clear policy, and render layers; the legacy `ViewportCameraSnapshot` is a projection for callers that have not yet been migrated.

`RenderViewExtract::selected_camera_descriptor()` selects the descriptor matching `scene_camera_entity` and falls back to the first descriptor for synthetic or explicit camera extracts. `selected_camera_target()`, `selected_camera_layers()`, and `selected_effective_camera()` are the submit/visibility read path for descriptor-owned fields. Plan 09 CO-M4 also exposes `selected_camera_volume_layers()` as the post-process Volume read path: visibility and draw culling keep using `selected_camera_layers()`, while `PostProcessExtract::resolved_settings_for_camera(...)` must receive only the selected descriptor `volume_mask`. `sync_selected_descriptor_camera_payload()` exists only for the transition window: it copies the current snapshot payload fields such as transform, projection, dynamic resolution, MSAA, and temporal jitter into the selected descriptor. It does not reproject target, viewport, order, clear, or layer data back onto `ViewportCameraSnapshot`; those fields no longer exist on the snapshot. Descriptor selection also does not project selected-descriptor MSAA back into `RenderViewExtract::anti_alias`, so explicit view AA such as TAA remains stable across camera-stack selection. `apply_target_size(...)` runs that sync before applying the descriptor viewport clamp and writing the sized aspect ratio into the selected descriptor payload. `render_view_apply_target_size_preserves_descriptor_target_and_layers` covers that transition by asserting a headless descriptor target, layer masks, volume mask, and viewport-derived aspect ratio survive sizing.

Status anchor `render_plan09_volume_mask_separate_from_culling_static_passed_cargo_lock_blocked_timeout_no_result` records the 2026-06-23 separation: scene post-process extraction now collects volumes for the selected/stack `volume_mask`, and submit-time resolution no longer reuses the camera culling mask.

The size is derived from the selected descriptor's explicit viewport rectangle or headless camera target when present, then falls back to the known submission target size. `RenderFrameExtract::apply_viewport_size(...)` updates both the selected descriptor payload aspect ratio and the stored target size before submission.

Plan 09 CO-M4 now keeps visibility renderable layer input typed through the frame DTO. `VisibilityRenderableInput.render_layer_mask` is a `RenderLayerSet`; `RenderFrameExtract::from_snapshot(...)` clones the typed mesh snapshot layer set instead of projecting through `to_legacy_mask_lossy()`, and `render_frame_extract_visibility_input_preserves_layers_above_legacy_mask_width` locks layer 40 through the snapshot adapter. The 2026-06-23 status anchor is `render_plan09_visibility_renderable_input_layer_set_static_passed_cargo_lock_blocked_timeout_no_result`; its focused locked Cargo test timed out after 124 seconds with no test binary, while locked check was blocked before compilation by current `Cargo.lock` drift. The inline `frame_extract.rs` tests were moved to `frame_extract/tests.rs`, keeping the production owner below the large-file threshold.

The 2026-06-24 Plan 09 frame extract geometry owner split moved mesh phase input, `GeometryExtract`, `StaticMeshBatchExtract`, static mesh batch keys, and static batching from `frame_extract.rs` into `frame_extract/geometry.rs`. The root re-exports `GeometryExtract`, `GeometryPhaseInput`, and `StaticMeshBatchExtract` from that child owner, so existing `frame_extract::{...}` callers keep the same public path. The later directory cutover makes `frame_extract.rs` a structural facade: `frame.rs` owns the neutral submission DTO and snapshot adapter, `scene_payload.rs` owns the immutable scene-generation aggregate, `shared_scene_domain.rs` owns independent Arc/COW domains, `view.rs` owns camera projection, `post_process.rs` owns post-process construction and resolution, and the particle, visibility, lighting, sprite, and base DTO owners remain separate leaves. `runtime_15_frame_extract_geometry_is_child_owner` continues to guard the geometry child mount and documentation anchors. Status anchor: `render_plan09_frame_extract_geometry_owner_split_static_passed_cargo_deferred_active_compile_lane`; the historical Plan 09 validation record remains unchanged, and current Cargo/WGPU/RenderDoc acceptance is still deferred.

`RenderViewExtract::effective_view_size()` is the canonical read path for SRP and RenderGraph descriptor derivation. It clamps through the camera viewport when present and falls back to `1 x 1` only when the extract does not yet know a surface or headless target size.

During submit, `build_frame_submission_context(...)` resolves the camera target against the viewport record and, for `RenderCameraTarget::Texture`, the referenced `TextureAsset` metadata before cloning the extract with `apply_viewport_size(...)`. Valid texture targets must be nonzero 2D single-layer single-mip descriptors with `RenderImageUsage::RenderTarget` and a renderable RGBA8 format; their descriptor extent becomes the submission size just like a headless target's explicit size. `RenderCameraTargetResolutionReport` then records the target kind, primary viewport size, resolved target size, effective view size, and dynamic-resolution-scaled render size into `RenderStats` and `render.camera.target.*` diagnostics.

The same context also resolves a crate-internal `ViewportRenderOutputTarget` and attaches it to renderer-bound `ViewportRenderFrame` values. Generated extract submits and direct runtime-frame submits both carry the resolved target kind, headless size, or texture handle plus size after preflight. `ViewportRenderFrame::camera()` now returns the selected `CameraRenderDescriptor`; renderer code that still needs a legacy `ViewportCameraSnapshot` for matrix math, shadows, particles, or post-process uniforms must call `ViewportRenderFrame::effective_camera()` and treat the returned snapshot as a projection. This keeps texture writeback and target-aware capture from revalidating authored camera state while `RenderFrameExtract` remains neutral and still carries no WGPU surface or texture object.

`ViewportRenderOutputTarget::writeback_plan(...)` is the renderer-internal planning seam. Non-texture targets report `NotRequested`; texture targets without a descriptor format report `PendingTargetDescriptor`; texture targets matching the framework offscreen output format label `rgba8unorm_srgb` report `ReadyForSrgbCopy`; linear `rgba8unorm` targets report `ReadyForConversion`; unsupported target formats report `BlockedFormatMismatch`; and prepared WGPU resources whose descriptor format no longer matches the camera-target preflight result report `BlockedPreparedFormatMismatch` before copy or conversion. The plan records texture handle, resolved size, source format, actual prepared target format, and expected preflight target format without exposing resource-streamer texture internals.

`ResourceStreamer::ensure_scene_resources(...)` notices the resolved texture output target on `ViewportRenderFrame` and asks a dedicated output-target residency path to prepare that texture. Primary-surface and headless targets are ignored. Output-target residency uses `OutputTargetTextureResource` instead of the sampled material/sprite `GpuTextureResource`, so a valid camera target may be render-target-only without also needing sampled binding usage. When an sRGB target is prepared and the graph-import plan is ready, the renderer imports the prepared texture view as the final graph target aliases instead of binding those aliases to the framework offscreen final color. Linear `rgba8unorm` targets still use `ResourceStreamer::execute_output_target_writeback(...)` after graph execution: the fullscreen conversion pass samples the framework final color and writes the prepared linear target. The writeback report records skipped-direct-import, suppressed-by-camera-stack, blocked, ready, copied, or converted status plus target extent, copy/conversion counts, and separate copy/conversion debug-marker emission. Prepared texture internals and WGPU handles stay behind the renderer resource owner instead of entering the frame DTO.

The renderer also records a separate graph-import report from that prepared output target. `ViewportRenderOutputTarget::graph_import_plan(...)` marks matching `rgba8unorm_srgb` texture targets as `ReadyForDirectImport` during residency/readiness preflight, linear `rgba8unorm` targets as `RequiresConversionWriteback`, unsupported formats as `BlockedFormatMismatch`, prepared-format drift as `BlockedPreparedFormatMismatch`, and non-texture targets as `NotRequested`. Graph execution upgrades successful sRGB imports to `DirectImported`, increments `direct_import_count`, and makes output-target writeback report `SkippedDirectImport`; readiness-only reports keep `direct_import_count` at zero. Both internal blocked states project to the neutral `blocked_format_mismatch` report row, so diagnostics stay stable while the planner keeps cache-key/preflight format identity aligned with the actual resource descriptor. If the selected child frame is not the stack-terminal final-output owner, `select_final_target_output(...)` records `SuppressedByCameraStack` for texture targets instead of importing the prepared target. `RenderStats.last_camera_target_graph_import` and `render.camera.target.graph_import.*` expose those status/count/extent rows without moving WGPU texture handles into `RenderFrameExtract`.

`ViewportFrame` now carries `RenderCaptureReport` from the renderer readback path into `record_capture(...)`, and `CapturedFrame` stores the same report in the viewport record. `record_submission(...)` then forwards the stored report into `SubmissionRecordUpdate`, so `update_base_stats(...)` can publish `RenderStats.last_capture_report`. This is intentionally result metadata: it records whether the capture came from framework offscreen color, an imported texture target, a converted texture writeback, or a copied texture writeback, while `RenderFrameExtract` remains the authored-frame DTO and still carries no backend texture handle.

## Temporal History Handoff

The submit context carries renderer-private temporal camera state derived from previous successful submissions rather than from editor state. `ViewportRecord` stores the previous camera snapshot separately from color-history validation; `build_frame_submission_context(...)` copies that camera snapshot into `FrameSubmissionContext`, and `build_runtime_frame(...)` attaches it to the renderer-bound `ViewportRenderFrame`. Successful submit and present paths record `ViewportRenderFrame::effective_camera()` with temporal jitter cleared, so camera velocity, scene uniforms, particle previous billboard bases, depth-of-field preparation, and velocity-camera post-process passes all consume the same selected descriptor projection instead of reaching back to raw `extract.view.camera`.

Object previous transforms no longer travel through the neutral frame DTO. The renderer-owned GPUScene rolls current instance transforms into `prev_world_from_local` only after successful WGPU submission, and `build_mesh_draws` reads that rolled previous transform when preparing temporal velocity draw eligibility. This keeps `RenderFrameExtract` focused on authored frame data while object velocity history stays with the shader-visible scene-data owner.

## Particle State Contract

`ParticleExtract` now separates current transparent billboard state from optional previous-state evidence. `sprites` carries the current `RenderParticleSpriteSnapshot` list used by the transparent particle pass. The current sprite snapshot's `render_layer_mask` is a typed `RenderLayerSet`; scene extraction wraps the legacy authored entity mask at this DTO boundary, while visibility input still downgrades through `to_legacy_mask_lossy()` because `VisibilityRenderableInput` remains on the legacy `u32` ABI. `previous_sprites` carries `RenderParticlePreviousSpriteSnapshot` rows keyed by entity, with previous position, size, aspect ratio, billboard offset, and rotation. The scene JSON extraction path initializes `previous_sprites` as empty because it does not yet own previous particle state.

Plan 09 CO-M4 also routes the particle selected-camera filter through that typed mask: `build_particle_vertices(...)`, `build_particle_velocity_vertices(...)`, and `RenderPipelineAsset::compile(...)` intersect the selected camera `RenderLayerSet` with each particle sprite `render_layer_mask`. The status anchor for this follow-up is `render_plan09_particle_render_layer_set_snapshot_static_passed_cargo_lock_blocked`; scoped rustfmt/static checks passed, but the focused locked Cargo command stopped before compilation on current `Cargo.lock` drift.

`ParticleExtract::previous_state_sprite_count()` matches current sprite entities against previous-state rows with per-entity counts, consuming one previous row for one current sprite. `missing_previous_state_sprite_count()` reports the remaining current sprites. Submit stats use that count to avoid marking particles as missing velocity input once a caller supplies matched previous state. This is still a neutral DTO contract and diagnostic contraction; it does not write particle velocity into `scene-velocity`.

## Scene Camera Scheduling Metadata

`RenderViewExtract` also carries optional scene camera provenance: `scene_camera_entity` and `scene_camera_order_report`. Scene-backed producers fill these fields after running render-extract systems so later multi-camera scheduling, diagnostics, and editor overlays can see the same active-camera ordering evidence as the scene. The report's `SortedRenderCamera` rows include the projected `CameraRenderDescriptor` plus the descriptor-backed `render_type`, so diagnostics and future camera-loop code can distinguish Base/Overlay provenance without asking the scene world again.

Plan 09 adds `RenderViewExtract.cameras` as the extract-side multi-camera descriptor list. Synthetic extracts, snapshot adapters, and explicit `SceneViewportExtractRequest::camera` overrides keep a single descriptor with no scene entity. Scene-backed extracts write all active scene cameras as `CameraRenderDescriptor` rows in deterministic scheduling order, and the selected scene camera descriptor is aligned to the effective `view.camera` payload after request projection or viewport-size overrides. `RenderViewExtract::selected_camera_descriptor()` is the descriptor read path for code that needs the selected scene/synthetic camera facts. `RenderFrameExtract::apply_viewport_size(...)` only synchronizes the selected scene descriptor, or the synthetic single descriptor, so non-selected Texture/Headless descriptors keep their own authored target and viewport. Scene extraction, visibility custom-target construction, submit preflight, history validation, renderer-bound frame access, and the offscreen submit camera loop now consume this descriptor list for target/layer/order ownership instead of reading custom-target facts from `RenderCameraOrderReport` or raw snapshot fields.

Offscreen WGPU submit resolves `view.cameras` through `resolve_camera_sequence(...)` in `submit_frame_extract/submit/camera_loop.rs`, flattens each Base camera followed by its Overlay descriptors, and builds a child `RenderFrameExtract` for each descriptor with `RenderFrameExtract::with_selected_camera_descriptor(...)`. The child extract replaces the active descriptor list with that one selected descriptor, updates the transitional `view.camera`, core-pipeline kind, MSAA settings, target size, and selected scene entity, then runs the existing single-camera submit body. The loop attaches the shared UI extract only to the terminal child of the last `PrimarySurface` Base stack, falling back to the terminal child of the last Base stack when no primary stack exists. This is the current M1-S2 loop scaffold: it provides true per-descriptor offscreen submit iteration under one render-framework operation lock, but it does not yet implement Base/Overlay attachment reuse, load-op translation, final-target composite semantics, or per-camera post/history/light ownership.

During WGPU submit the report is copied into `FrameSubmissionContext` and projected into `RenderStats.last_scene_camera_scheduled_count` plus `RenderStats.last_scene_camera_order_ambiguity_count`, then into `render.camera.scheduled_count` and `render.camera.order_ambiguity_count` diagnostics. The report remains scheduling and visibility metadata. Generated offscreen submit, native surface present, and direct runtime-frame submit now consume descriptors by projecting one selected descriptor at a time into the renderer path. The outer loop separately records `RenderStats.last_camera_loop_submission_count` only after the complete sequence succeeds and projects it to `render.camera.loop_submission_count`; this execution count includes derived planar capture cameras and must not be inferred from the optional scene-order report. Final custom-target composite, complete split-screen product evidence, and per-camera post/history/light ownership remain separate renderer milestones.

## Sort Key Contract

`RenderPhase::queue_order()` is the public cross-phase submission precedence used before a phase-local sort key is compared. The order is prepass, shadow, opaque 2D/3D, alpha-mask 2D/3D, deferred, transparent 2D/3D, post-process, UI, overlay, then debug. Matching 2D/3D geometry phase orders are intentional: the product pipeline chooses the concrete pass while the queue contract keeps phase precedence stable for diagnostics and renderer tooling. `RenderPhase::diagnostic_name()` exposes stable kebab-case labels such as `opaque-3d`, `transparent-2d`, and `post-process` for logs, editor displays, and RenderDoc/export labels without requiring each consumer to mirror the phase enum.

`RenderPhaseItem::ordering_key()` returns `RenderPhaseQueueOrderingKey`, the exact tuple consumed by `RenderPhaseQueue`: phase order, packed phase-local sort key, then entity id. The key also exposes `raw_sort_key()` for labels and diagnostics, but its `Ord`/`Eq` implementation stays identical to the queue tuple so editor ordering views cannot drift from submission order.

`RenderPhaseQueue::summary()` returns `RenderPhaseQueueSummary` for an already-built queue. It records total item count, named `RenderPhaseQueueSummaryPhaseCount` rows in `RENDER_PHASES_BY_QUEUE_ORDER`, phase-order spans, and the first and last `RenderPhaseQueueOrderingKey` values. Each phase-count row carries phase, owned `diagnostic_name`, phase order, and item count, with `phase_count_row_for_phase(...)` as the direct lookup helper, while each span records the phases represented by that shared order bucket, its owned joined `diagnostic_name`, the sorted queue index range, and first/last ordering keys. `span_for_phase(...)` maps a concrete phase such as `Opaque3d` back to its shared phase-order bucket, and `span_for_queue_index(...)` maps a sorted queue item index back to the bucket range that contains it. `active_phase_counts()` and `active_phase_order_spans()` stream only rows whose queue counts are nonzero for compact diagnostics, while the stored vectors preserve zero-count rows for stable table layouts. `RenderPhaseQueueSummaryPhaseCount::diagnostic_name()` returns a borrowed view of the stored phase label, while `RenderPhaseQueueSummaryPhaseOrderSpan::diagnostic_name()` returns a borrowed view of the stored shared-bucket label, such as `opaque-2d+opaque-3d` or `transparent-2d+transparent-3d`. Serialized diagnostics can therefore display both per-phase rows and bucket groups without rescanning or resorting queue items. The queue summary and its child rows derive `Serialize`/`Deserialize`, and the serialized payload carries those owned `diagnostic_name` fields directly. `GeometryExtract::phase_queue_summary()` and `SpriteExtract::phase_queue_summary()` expose the same reporting view directly from frame DTOs by deriving it from the current sorted `phase_queue`; they do not cache a second copy of queue state.

`RenderFrameExtract::phase_queue_summary()` returns `RenderFramePhaseQueueSummary`, whose DTOs and lookup helpers live in `frame_phase_queue_summary.rs`, while the frame DTO and adapter live in `frame_extract/frame.rs` behind the `frame_extract.rs` facade. The frame summary carries the mesh and sprite summaries side by side plus total, per-phase, and per-phase-order bucket counts across both queues. The frame summary preserves the full child queue bounds through geometry and sprite first/last ordering keys, so diagnostics can inspect the top-level mesh and sprite ordering ranges before drilling into individual buckets. Its `phase_counts` table is ordered by `RENDER_PHASES_BY_QUEUE_ORDER`; each `RenderFramePhaseQueueSummaryPhaseCount` row records the phase, owned `diagnostic_name`, phase order, geometry count, sprite count, and combined total, with `phase_count_row_for_phase(...)` as the direct lookup helper and a borrowed `diagnostic_name()` label accessor. Its `phase_order_spans` table records one `RenderFramePhaseQueueSummaryPhaseOrderSpan` per shared phase-order bucket, including the represented phases plus an owned joined `diagnostic_name`, geometry, sprite, and total counts, and the span exposes the same borrowed stored `diagnostic_name()` label as the child queue summary. `active_phase_counts()` and `active_phase_order_spans()` provide compact nonzero frame rows for editor panels, runtime logs, and future RenderDoc label exporters without losing the stable full-table vectors. Each frame bucket also preserves the child queue evidence for that bucket: mesh and sprite start index, exclusive end index, first ordering key, and last ordering key. `phase_order_span_for_phase_order(...)`, `phase_order_span_for_phase(...)`, `phase_order_span_for_geometry_queue_index(...)`, and `phase_order_span_for_sprite_queue_index(...)` let diagnostics jump directly to buckets such as opaque 2D/3D or transparent 2D/3D without recomputing bucket membership or rescanning child queues. The frame summary and its child rows derive `Serialize`/`Deserialize`, matching the child `RenderPhaseQueueSummary` payloads so runtime diagnostics, editor panels, and future RenderDoc label exporters can persist exactly the same bucket evidence they inspect in memory. This is intentionally a reporting view over the sorted queues, not a second grouping or ordering path.

`RenderPhaseSortKey` now exposes `RenderPhaseSortComponents` as the shared ordering input for 3D, 2D, UI, overlay, and debug draw records. The packed order is render queue, material queue, order in layer, UI z-index, depth or reverse depth for transparent phases, then entity tie-breaker.

`RenderPhaseSortKey::breakdown(...)` returns `RenderPhaseSortKeyBreakdown`, a read-only diagnostic view of the same inputs and derived depth keys used by the packed sorter. It records the phase, queue fields, raw depth, depth bias, effective depth, finite-depth quantized key, transparent back-to-front reversal, entity tie-breaker, the low 16-bit packed entity tie-breaker key, and raw packed sort key. Each packed lane also exposes its derived `*_sort_key` value, so diagnostics can show both the authored value and the actual saturated or biased value that participates in the packed key. This gives editor panels, runtime diagnostics, and future RenderDoc labels an explainable render-order contract without adding a second sorting path.

`RenderPhaseSortKeyBreakdown::first_difference(...)` returns `RenderPhaseSortDecision`, whose `field` identifies the first lane that differs using the same order as `RenderPhaseQueue`: phase order, packed render queue, packed material queue, packed order in layer, packed UI z-index, packed ordered depth, packed entity tie-breaker key, then the full entity tie-breaker used after raw key equality. The decision reports `left_value`, `right_value`, and `left_before_right` for diagnostics while computing the direction from the actual packed lane values, so saturated queue fields do not create false explanations.

`GeometryPhaseInput`, `SpritePhaseExtractInput`, `MeshPhaseInput`, and `SpritePhaseInput` carry the same queue fields with defaulting constructors. Meshes use depth plus entity tie-breaker by default; sprites map `z_order` to order in layer and can now add material queue, render queue, depth bias, and UI z-index without changing the queue builder contract.

## Static Mesh Batch Extract

`GeometryExtract` carries `static_batches` alongside the mesh vector and phase
queue. The batch list is derived automatically by
`GeometryExtract::from_meshes(...)` and
`GeometryExtract::from_meshes_and_phase_inputs(...)`: only meshes whose
`mobility` is `Mobility::Static` are eligible, and they are grouped by source
model, optional mesh primitive handle, material handle, and typed render-layer
set. The internal batch key stores the layer set as an ordered layer list, so
static batching does not collapse layer 32+ meshes through a lossy legacy mask.
Groups with one instance are omitted so downstream renderers can treat the list
as actual batch candidates rather than a mirror of every static mesh.

`StaticMeshBatchExtract` records the resource key, typed `render_layer_mask`,
the source `mesh_indices`, and the entity ids in deterministic mesh-vector
order. It is still neutral frame data: it does not store renderer buffers, WGPU
bind groups, or draw commands. The immediate consumer is diagnostics,
frame-history validation, and acceptance testing, while a later renderer pass
can use the same DTO to emit instanced/static draw calls without recomputing
scene ownership from `World`. The remaining legacy conversions are explicit:
`RenderFrameExtract::from_snapshot(...)`, scene `build_visibility_input(...)`,
and visibility fallback rows call `to_legacy_mask_lossy()` only when exporting
to the old `VisibilityRenderableInput` / visibility ABI.

Plan 09 CO-M4 routes mesh selected-camera filtering through the same typed mask:
`RenderMeshSnapshot.render_layer_mask` is a `RenderLayerSet`, and
`build_mesh_draws/build/phase_ordering.rs` intersects selected camera layers
with that set for raw mesh-vector fallback and `RenderPhaseQueue` consumption.
The status anchor is
`render_plan09_mesh_render_layer_set_snapshot_static_passed_cargo_lock_blocked`;
scoped rustfmt/static checks passed, but the focused locked Cargo command
stopped before compilation on current `Cargo.lock` drift.

The 2026-06-25 Plan 02 virtual-geometry debug snapshot contract repair keeps
manual `RenderMeshSnapshot` fixture construction aligned with that typed layer
contract. `virtual_geometry_debug_snapshot_contract.rs` now converts the
legacy scene default through `RenderLayerSet::from_legacy_mask(...)` before
assigning `render_layer_mask`, so debug snapshot tests no longer reintroduce a
raw `u32` mask at the DTO boundary. Status anchor:
`render_plan02_vg_debug_snapshot_typed_layer_contract_cargo_check_passed`;
focused `cargo check` for that integration test target passed with existing
warning noise.

The vampire example relies on this path for its authored billboard grass:
six `Static Grass Batch ...` entities share the same grass model/material and
are expected to collapse into one runtime static batch in the frame extract.

## Design And Rationale

The size belongs on the extract, not in the SRP asset, because the same pipeline asset can be used for multiple viewports, headless targets, editor previews, and camera stacks. The compiler therefore receives the product pipeline and per-frame view data separately and derives graph resource descriptors from both.

This is still neutral data. No WGPU surface, texture, or swapchain object is stored in the framework DTO.

## Test Coverage

The focused pipeline compile test verifies that a headless HDR camera with 4x MSAA produces `scene-color` and `scene-depth` graph lifetimes with the expected extent, format, and sample count. Broader scene extract and renderer execution validation remains part of the milestone testing stage.

Focused validation on 2026-06-02 passed for `pipeline_compile` with 42 tests, plus the two direct phase-order filters for mesh unified sort components and sprite material queue/UI z-index ordering. These runs used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain` and emitted only pre-existing warning classes outside this change.
