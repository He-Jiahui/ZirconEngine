---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/extract_registration.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/pending_completion/mod.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/pending_completion/apply_scene_prepare_resources.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/prepare_frame/build_scene_prepare_frame.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/scene_frame.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_cell.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_clipmap.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/test_accessors.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/input_set.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/representation.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/surface_cache_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/voxel_scene_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/hybrid_gi/build_hybrid_gi_scene_prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/update_hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/card_capture_shading.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_bind_group.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/queue_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/readback/hybrid_gi_gpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/hybrid_gi_gpu_pending_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_hybrid_gi_buffers/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_temporal_signature.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_history.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/scene_prepare_surface_cache_samples.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_surface_cache.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_representation.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/extract_registration.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/pending_completion/mod.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/pending_completion/apply_scene_prepare_resources.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/prepare_frame/build_scene_prepare_frame.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/scene_frame.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_cell.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_clipmap.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/test_accessors.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/input_set.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/representation.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/surface_cache_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/voxel_scene_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/hybrid_gi/build_hybrid_gi_scene_prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/update_hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/card_capture_shading.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_bind_group.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/queue_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/readback/hybrid_gi_gpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/hybrid_gi_gpu_pending_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_hybrid_gi_buffers/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_temporal_signature.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/scene_prepare_surface_cache_samples.rs
plan_sources:
  - user: 2026-04-21 continue Hybrid GI / Lumen-style implementation and keep advancing the approved three-phase plan
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_surface_cache.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_representation.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - cargo test -p zircon_runtime --locked scene::tests::world_basics::render_extract_separates_directional_point_and_spot_lights -- --exact
  - cargo test -p zircon_runtime --locked asset::tests::assets::scene::scene_asset_toml_roundtrip_preserves_point_and_spot_lights -- --exact
  - cargo test -p zircon_runtime --locked core::framework::tests::render_frame_extract_roundtrip_preserves_split_light_lists -- --exact
  - cargo test -p zircon_runtime --locked core::framework::tests::hybrid_gi_extract_defaults_to_public_settings_and_empty_internal_fixture -- --exact
  - cargo test -p zircon_runtime --locked graphics::tests::hybrid_gi_scene_representation::hybrid_gi_input_contract_stays_complete_for_deferred_and_forward_plus -- --exact
  - cargo test -p zircon_runtime --locked exact_runtime_ -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_resolve_blends_nonzero_exact_ -- --nocapture
  - cargo test -p zircon_runtime --locked page_table_and_capture_slots -- --nocapture
  - cargo test -p zircon_runtime --locked reuses_surface_cache_slots_after_invalidation -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_scene_representation_allocates_page_ids_separately_from_owner_card_ids -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_scene_representation_reuses_recycled_page_id_for_new_owner_after_invalidation -- --nocapture
  - cargo test -p zircon_runtime --locked card_capture_requests -- --nocapture
  - cargo test -p zircon_runtime --locked --lib encode_hybrid_gi_probes_uses_atlas_only_scene_prepare_card_capture_resources_for_hierarchy_irradiance -- --nocapture
  - cargo test -p zircon_runtime --locked --lib encode_hybrid_gi_probes_prefers_capture_scene_prepare_card_capture_resources_for_hierarchy_irradiance -- --nocapture
  - cargo test -p zircon_runtime --locked --lib surface_cache_page_truth_changes -- --nocapture
  - cargo test -p zircon_runtime --locked --lib current_surface_cache_truth_when_trace_schedule_is_empty -- --nocapture
  - cargo test -p zircon_runtime --locked --lib exact_runtime_irradiance_blends_current_surface_cache_truth_when_trace_schedule_is_empty -- --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_runtime::hybrid_gi_runtime_state_builds_scene_surface_cache_irradiance_continuation_without_trace_schedule -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::scene::scene_renderer::post_process::resources::execute_post_process::encode_hybrid_gi_probes::hybrid_gi_hierarchy_irradiance::tests::exact_runtime_irradiance_skips_scene_prepare_reblend_when_runtime_source_is_already_scene_driven -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_runtime::hybrid_gi_runtime_state_builds_scene_voxel_rt_lighting_continuation_without_trace_schedule -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_runtime::hybrid_gi_runtime_state_reports_higher_scene_truth_quality_for_voxel_rt_than_surface_cache_only_rt -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_runtime::hybrid_gi_runtime_state_reports_clean_surface_cache_scene_truth_freshness_above_dirty_surface_cache_truth -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_runtime::hybrid_gi_runtime_state_reports_clean_voxel_scene_truth_freshness_above_dirty_voxel_truth -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::scene::scene_renderer::post_process::resources::execute_post_process::encode_hybrid_gi_probes::hybrid_gi_hierarchy_rt_lighting::tests::exact_runtime_rt_lighting_skips_scene_prepare_reblend_when_runtime_source_is_already_scene_driven -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib exact_runtime_rt_with_current_surface_cache_truth -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_pending_probe_gpu_irradiance_blends_exact_runtime_source_with_current_surface_cache_truth_when_trace_schedule_is_empty -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_card_capture_requests_quantize_scene_prepare_requests -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_voxel_clipmaps_quantize_scene_prepare_clipmaps -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_prepare_descriptors_include_runtime_voxel_cells -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_prepare_descriptors_pack_explicit_card_capture_seed_rgb -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_prepare_descriptors_preserve_explicit_black_card_capture_seed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_radiance_changes_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_changes_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_matches_different_card_capture_seed_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_card_capture_material_seed_changes_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib collect_inputs_preserves_scene_prepare_contract_for_renderer_consumption -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_runtime_state_builds_scene_prepare_voxel_cells_from_scene_representation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_uses_runtime_voxel_cell_payload_without_scene_meshes -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_requires_runtime_voxel_cells_for_occupancy_and_count_truth -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_card_capture_samples_change_with_material_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_samples_change_with_material_emissive -- --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_scene_prepare_resources -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_absent_persisted_surface_cache_page_contents_do_not_create_resource_snapshot_without_other_scene_prepare_data -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_absent_persisted_surface_cache_page_contents_do_not_occupy_atlas_or_capture_slots -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_atlas_only_persisted_surface_cache_page_contents_do_not_occupy_capture_slots -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_capture_only_persisted_surface_cache_page_contents_do_not_occupy_atlas_slots -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_prepare_descriptors_skip_absent_clean_frame_persisted_surface_cache_pages -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_persisted_page_card_capture_seed_rgb_uses_atlas_when_capture_sample_is_absent -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_runtime_state_keeps_atlas_only_surface_cache_page_samples_across_clean_frames -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_runtime_state_keeps_capture_only_surface_cache_page_samples_across_clean_frames -- --nocapture
  - cargo test -p zircon_runtime --locked --lib scene_prepare_card_capture_request_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib scene_prepare_persisted_surface_cache_page_samples_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_occupancy_changes_with_mesh_translation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_samples_follow_mesh_translation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_occupancy_counts_accumulate_overlapping_meshes -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_dominant_node_prefers_brighter_overlap -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_dominant_sample_matches_brighter_overlap -- --nocapture
  - cargo test -p zircon_runtime --locked --lib scene_voxel_clipmap_occupancy_mask_moves_when_mesh_crosses_cells -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_readback_reports_scene_prepare_card_capture_resource_snapshot -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_card_capture_requests_move_near_or_far_from_probe -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_clipmaps_move_near_or_far_from_probe -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_cells_move_near_or_far_from_probe -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_runtime_state_builds_scene_prepare_frame_from_scene_representation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_resolve_uses_runtime_scene_voxel_point_light_seed_when_layout_and_tint_stay_fixed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_resolve_uses_runtime_scene_voxel_tint_when_layout_stays_fixed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_resolve_uses_runtime_scene_voxel_owner_card_capture_seed_when_layout_and_owner_stay_fixed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib scene_prepare_present_black_voxel_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib spatial_fallback -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_scene_representation -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_runtime_state_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_resources -- --nocapture
  - cargo check -p zircon_runtime --locked --lib
  - cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-gpu-runtime
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_gpu_runtime_source --target-dir target/codex-hybrid-gi-gpu-runtime -- --nocapture
  - cargo test -p zircon_runtime --locked --lib exact_runtime_rt_ --target-dir target/codex-hybrid-gi-gpu-runtime -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_runtime_state_ --target-dir target/codex-hybrid-gi-gpu-runtime -- --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_rejects_history_when_scene_driven_exact_runtime_truth_changes --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_gives_scene_driven_exact_runtime_truth_more_history_reuse_than_continuation --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::scene::scene_renderer::post_process::resources::execute_post_process::encode_hybrid_gi_probes::encode::tests::encode_hybrid_gi_probes_scales_temporal_scene_truth_confidence_with_runtime_support --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::scene::scene_renderer::post_process::resources::execute_post_process::encode_hybrid_gi_probes::encode::tests::encode_hybrid_gi_probes_accumulates_temporal_scene_truth_confidence_across_sources --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_gives_clean_surface_cache_scene_truth_more_history_reuse_than_dirty_surface_cache_truth --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_gives_clean_voxel_scene_truth_more_history_reuse_than_dirty_voxel_scene_truth --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_rejects_history_when_surface_cache_scene_truth_freshness_changes_without_rgb_change --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_rejects_history_when_voxel_scene_truth_freshness_changes_without_rgb_change --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_history --target-dir target/codex-hybrid-gi-history
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_runtime --target-dir target/codex-hybrid-gi-history
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_surface_cache --target-dir target/codex-hybrid-gi-history
  - cargo test -p zircon_runtime --locked --lib encode_hybrid_gi_probes --target-dir target/codex-hybrid-gi-history
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_uses_descendant_scene_driven_runtime --target-dir target/codex-hybrid-gi-history
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_gathers_requested_descendant_runtime --target-dir target/codex-hybrid-gi-history
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_blends_nonzero_exact --target-dir target/codex-hybrid-gi-history
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_surface_cache::hybrid_gi_resolve_rejects_global_illumination_history_when_surface_cache_page_truth_changes --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_uses_descendant_scene_driven_runtime_irradiance_for_parent_probe_after_schedule_clears --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_uses_descendant_scene_driven_runtime_rt_for_parent_probe_after_schedule_clears --target-dir target/codex-hybrid-gi-history -- --exact --nocapture
  - cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-history
doc_type: module-detail
---

# Hybrid GI Lumen-Style Scene Representation

## Purpose

这份文档记录 `Hybrid GI / Lumen-Style V1` 当前已经落地到 `zircon_runtime` 的第一阶段切口。重点不是最终 GI 质量，而是把“公共 extract 合同”和“renderer/runtime 内部 scene representation 真源”开始分开。

当前这轮实现只推进到 milestone 1 的基础层：

- 通用 scene extract 已经扩成 `directional + point + spot`
- `RenderHybridGiExtract` 已经收口成 public settings / budget / debug payload
- renderer/runtime 内部已经有独立的 `HybridGiSceneRepresentation / HybridGiSurfaceCacheState / HybridGiVoxelSceneState / HybridGiInputSet`
- cards、surface cache、voxel clipmap 的状态机开始进入内部权威状态，而不是继续把 authored probe / trace-region 当长期真源

## Public Contract Cutover

### Split Light Scene Extract

`RenderSceneGeometryExtract` 不再把所有灯型塞进单一 `lights` 列表，而是显式拆成：

- `directional_lights`
- `point_lights`
- `spot_lights`

这一步是后续 clustered direct-light injection 和 surface-cache direct-light seed 的前置条件。scene/world roundtrip 与 frame extract roundtrip 也已经一起升级，所以 scene authoring、asset roundtrip、frame contract 现在看到的是同一套 split-light 语义。

### Hybrid GI Public Extract Now Carries Settings Only

`RenderHybridGiExtract` 的公开面已经改成：

- `enabled`
- `quality`
- `trace_budget`
- `card_budget`
- `voxel_budget`
- `debug_view`

旧的 `probe_budget / tracing_budget / probes / trace_regions` 仍然临时存在，但只保留为 crate-internal fixture bridge，用来维持当前 runtime host 与旧测试夹具的迁移期可编译性。它们不再代表长期 authoring API，也不应该再被上抬成 render façade 的正式场景输入。

## Internal Scene Representation State

### Shared GI Input Contract

`HybridGiInputSet` 现在固定了 Lumen-style GI 需要的最小输入集：

- depth
- normal
- roughness
- base color
- emissive
- history validity
- motion vectors

当前实现把 Deferred 和 Forward+ 都约束到同一份输入合同上。Deferred 直接喂完整 GBuffer；Forward+ 的最终目标是补一套 `GBuffer-lite` 附件来满足同一 contract，而不是另做第二套 GI 算法。

### Scene Representation Skeleton

`HybridGiSceneRepresentation` 当前已经成为 runtime host 内部的聚合状态，负责持有：

- public settings mirror
- registered card descriptors
- surface cache state
- voxel scene state
- fixed GI input contract

这意味着 `Hybrid GI` 已经开始从“纯 probe runtime cache”向“scene-driven internal representation”过渡，哪怕当前 cards 还没有完全从真实 renderer scene registration 自动派生。

这轮继续往前推进后，这个描述已经不再只是骨架声明。`build_hybrid_gi_runtime(...)` 现在会把当前 frame 的真实 `meshes + directional/point/spot lights` 一起送进 runtime host，`HybridGiRuntimeState` 不再只消费 `RenderHybridGiExtract` 的 settings。

## Milestone 1 State Behavior

### Surface Cache Budgeting

`HybridGiSceneRepresentation::synchronize_cards(...)` 目前提供了第一版 deterministic card-state 迁移逻辑：

- scene 中 active card id 会被规范成稳定去重后的 card descriptor 列表
- `card_budget` 决定本帧可 resident 的 surface-cache page 数
- 已 resident 且仍然有效的 page 会优先保留
- 新进入 resident 的 page 会被标记为 dirty capture
- 超出 budget 的 card 会进入 feedback 列表，表示后续需要 capture / residency 机会
- 离开 active 集合或因为 budget 收缩而被挤出的 resident page 会进入 invalidation 列表

当前 surface-cache 还不是最终 atlas/page-table 实现，但已经具备 milestone 1 需要的最小状态机语义：注册、复用、失效、反馈。

这一段继续推进后，surface-cache 已经不再只是 resident/dirty 列表：

- resident page 现在会持有 deterministic `page table -> atlas slot` 映射
- dirty resident page 会持有 deterministic `card capture atlas slot` 映射
- page 继续 resident 时会尽量保留原 atlas slot
- invalidated page 会释放 atlas / capture 槽位，后续新 page 复用最低可用槽位
- resident page 的 capture slot reservation 现在也会独立保留，所以 page 在 clean frame 之后再次变 dirty 时仍会回到原 capture slot，而不是被重新压到最低空槽
- resident page allocation 现在已经和 owner card 显式解耦：runtime 会分配内部 `page_id`，从 `0` 开始递增，并在 invalidation 后优先复用已释放 id，而不再把 `page_id` 直接等同于 `card_id`
- `page_id` 现在只负责 residency / dirty / invalidation / slot lifetime，`owner_card_id` 才负责 bounds lookup、persisted page owner matching，以及 voxel / resolve 里的 surface ownership
- 如果同一轮 sync 里旧 page 先 invalidated 又立刻被新 owner 复用，同一个 `page_id` 可以同时出现在 invalidation 与新 resident 集合里；这里的 invalidation 语义明确表示“旧内容生命周期结束”，不是否认新 owner 已经接管该页

也就是说，Milestone 1 里的 `page table / atlas / card capture atlas` bookkeeping 已经开始落到 runtime 内部权威状态。renderer 这一侧现在也已经补上了第一版 per-frame GPU atlas / capture RT scaffold，而且不再只是“空纹理 + slot-truth”：当前已经会把 scene-driven request 写成第一版 scene-driven direct-light seed texel 内容并做 sample readback。只要 request 能解析到 matching mesh，renderer 就会直接消费当前 frame 的 `meshes + directional/point/spot lights`，并进一步经由材质解析拿到 `base_color + emissive`，再和 mesh tint 一起合成最小 capture radiance；只有解析不到 matching mesh 时才退回 deterministic debug texel。它仍然不是最终的 surface-cache shading pass。

最新这层又把材质真值拉进了同一条 seam，而不是继续停在 mesh-instance tint 代理：

- `collect_inputs(...)` 仍然只传 scene mesh / split-light snapshot，不新增 public extract DTO
- `SceneRendererCore::execute_runtime_prepare_passes(...)` 现在会把 `ResourceStreamer` 一起交给 `HybridGiGpuResources::execute_prepare(...)`
- card-capture 着色逻辑已经从 `create_buffers.rs` 拆到独立的 `card_capture_shading.rs`
- `ResourceStreamer` 现在能通过已准备好的 `MaterialRuntime`，或必要时回退到 `ProjectAssetManager::load_material_asset(...)`，解析 card capture 需要的 `base_color + emissive` 种子
- atlas / capture texel 的最小 radiance 现在由 `mesh tint * material base_color + material emissive + directional/point/spot direct-light seed` 共同决定，而不再只是 mesh tint 乘灯光
- `create_buffers.rs` 现在会在创建 atlas / capture 纹理之前就把同一份真实 texel 颜色写进 `scene_prepare_resources.atlas_slot_rgba_samples / capture_slot_rgba_samples`，因此这份 snapshot 不再只在 pending-readback collect 之后才有意义；它在当前 frame 的 post-process 阶段就已经能代表 authoritative card-capture seed truth
- 同一份 `scene_prepare_resources` snapshot 现在还会为每个 resident voxel clipmap 派生一条最小 `voxel_clipmap_rgba_samples` 调试样本，并额外记录 `voxel_clipmap_occupancy_masks`、`voxel_clipmap_cell_rgba_samples`、`voxel_clipmap_cell_occupancy_counts`、`voxel_clipmap_cell_dominant_node_ids` 与 `voxel_clipmap_cell_dominant_rgba_samples`。前者用 clipmap 包围内的 scene mesh/material/light 种子聚合成 deterministic radiance；occupancy 会把 mesh translation 粗量化进固定 `4x4x4 -> u64` occupancy grid；cell sample 会在同一固定 `4x4x4` grid 的每个 occupied cell center 上重用同一份 material/light 着色种子，形成 cell-level volume-content readback；cell count 会把重叠 mesh 对同一 coarse voxel cell 的占用次数直接压回 readback；dominant node id 则会把当前 cell 内 radiance 更强的 mesh authority 固定下来；dominant RGBA sample 则把这份更亮 contributor 自己的 radiance 颜色保留下来，从而和 aggregate cell sample 分离，方便在 Milestone 1 阶段同时验证 voxel scene 的 radiance seed、空间驻留、粗体素内容、cell-level residency density、coarse contributor ownership 与 authority color truth 都已经接到 scene-driven capture 链路

### Card Capture Request Descriptors

在这层 bookkeeping 之上，scene representation 现在还会继续派生一份真正面向 renderer seam 的 `card capture request` 描述，而不只是“有哪些 dirty page/capture slot”：

- 每条 request 都会同时带上 `card_id / page_id / atlas_slot_id / capture_slot_id`
- request 还会携带 card 当前的 `bounds_center / bounds_radius`
- request 集合只覆盖当前 dirty resident page，不会把 clean resident page 混进 capture 队列
- 当 resident page 保持不变、只有其中一张 card 再次变 dirty 时，request 会继续复用原 `atlas_slot_id + capture_slot_id`
- request 的 `card_id + bounds` 现在总是通过 resident `owner_card_id` 反查，而不是假定 `page_id == card_id`；因此 renderer seam 已经接受 “page lifecycle” 和 “scene owner semantics” 分层

这意味着 Milestone 1 现在不只是有 page-table/capture-slot bookkeeping，还已经把 “哪张 card 该被 capture 到哪个 atlas/capture slot，以及它当前代表的几何包围” 固定成内部真源。当前 renderer 创建 per-frame card-capture atlas / capture textures 时，已经直接消费这份 scene-driven descriptor，因此后续真正接入 capture shading pass 时，不需要重新发明一套 slot 对齐逻辑。

### Persistent Surface-Cache Page Content

这一轮继续把 `surface cache` 从“只有 slot bookkeeping 和每帧 renderer readback”往前推了一层：runtime 现在会按 resident `page_id` 持久保存最近一次 `scene_prepare` capture 的 atlas/capture 样本，而不是让这份真值在 frame 结束时直接蒸发。

- `collect_gpu_completions(...)` 不再丢掉 `HybridGiScenePrepareResourcesSnapshot`
- `update_hybrid_gi_runtime(...)` 现在会把这份 snapshot 回灌到 `HybridGiRuntimeState`
- `HybridGiSurfaceCacheState` 现在会把 persisted page content 显式拆成 `page_id + owner_card_id + atlas_slot_id + capture_slot_id + atlas/capture samples`，不再继续把 resident `page_id` 当成 owner card 的隐式别名
- clean frame 即使没有新的 dirty capture request，也会继续保留上一次已经写入的 page content
- resident page 的 slot reservation 如果保持不变，page content 也会跟着当前 slot 重新绑定，而不是被错误清空
- invalidated page 会同步移除对应的 persisted content；新 replacement page 在拿到第一次 capture readback 之前不会伪造内容
- recycled `page_id` 只有在当前 resident owner 仍然匹配旧 `owner_card_id` 时才会保留原 persisted texel；如果 page 被重新绑定到新 owner，旧内容会被丢弃并等待新的 dirty capture 重新填充
- `HybridGiScenePrepareFrame` 现在还会额外导出 `surface_cache_page_contents`，把这层 persisted page sample 作为 runtime-owned clean-frame scene truth 一起送进 renderer，而不是只在 runtime 内部留一份 CPU-side cache
- renderer `collect_inputs(...) / create_buffers.rs` 现在会把这批 persisted page sample 和当前 dirty `card_capture_requests` 合并成同一份 atlas/capture slot 占用与 RGBA readback；因此即使当前 frame 完全没有新的 dirty capture request，scene-prepare readback 仍然能携带有效的 atlas/capture sample
- 这批 clean-frame persisted page 现在还会继续上抬成 GPU completion 可见的 synthetic card descriptor：`collect_inputs(...)` 会把没有匹配 dirty request 且至少带有一份 present sample 的 resident page 计入额外的 `scene_card_capture_descriptor_count`，`create_buffers.rs` 会优先用 persisted `capture_sample_rgba`、其次回退到 `atlas_sample_rgba` 去补成 `SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE`，而 `queue_params(...)` 也会用这份 descriptor 数驱动 `scene_card_capture_request_count`
- 这条 persisted-page descriptor seam 现在也补上了和 resolve-side 同步的 presence contract：`alpha = 0` 的 atlas/capture 样本会被视为 truly absent，不再伪造成黑色 synthetic descriptor authority；显式黑色仍然通过 `alpha = 255, rgb = 0` 保持 authoritative
- `scene_prepare_resources(...)` 现在也把这条 presence contract贯彻到 renderer 资源层本身：occupied atlas/capture slots、slot counts、RGBA readback 和实际 atlas/capture texture/upload/sample-readback 都按 side 独立构建。atlas-only page 不会再伪造 capture-slot authority 或触发零尺寸 capture texture，capture-only page 也不会再反向占用 atlas side
- `HybridGiSurfaceCacheState::apply_scene_prepare_resources(...)` 现在允许 resident persisted page 只带 atlas 或只带 capture sample 继续存在；只要任一 side 的 `alpha > 0`，clean-frame runtime reuse 就会保留这一页的 one-sided truth，只有 atlas/capture 两边都 absent 时才会把该页视为 truly absent。空 snapshot 仍然会保留既有 clean-frame 内容，不会把 runtime surface cache 意外清空
- resolve-side owner-card fallback 也开始复用这条 clean-frame seam：当 runtime voxel `dominant_card_id` 仍然存在、但当前 frame 没有匹配 `card_capture_request` 时，`hybrid_gi_hierarchy_rt_lighting.rs` 会继续优先使用 persisted `surface_cache_page_contents`，而不是直接掉回空间启发式
- resolve-side 现在还补上了第一版 surface-cache spatial fallback：当 scheduled trace 为空、runtime voxel clipmap/cell 也没有有效支持时，`hybrid_gi_hierarchy_rt_lighting.rs` 会按 `card_capture_requests + surface_cache_page_contents` 的 `bounds_center / bounds_radius` 做近场 page/sample 混合，而不是直接掉回纯黑；dirty request 会优先覆盖同 page 的 persisted page sample
- 这条 request/page/sample authority 现在也被抽成了独立的 `scene_prepare_surface_cache_samples.rs` 模块：`card_capture_request` 资源采样、persisted page 采样、owner lookup、bounds-weighted spatial fallback，以及 `capture > atlas > synthetic` 与 `alpha = 0 => absent` 的 presence 语义不再在 RT lighting / irradiance 里各写一份
- `hybrid_gi_hierarchy_irradiance.rs` 现在也会在 exact/inherited/descendant runtime irradiance 与 ancestor prepare irradiance 全部缺席时复用同一条 persisted-page surface-cache fallback，所以 atlas-only 与 capture-preferred clean-frame page truth 不再只影响 RT lighting / final resolve，而是能继续驱动 hierarchy irradiance
- post-process encode 这一层现在也补齐了同样的不对称 seam：`encode_hybrid_gi_probes(...)` 不再只把 `scene_prepare_resources` 传给 hierarchy RT lighting，`hierarchy_irradiance_rgb_and_weight` 现在也会消费当前帧 dirty `card_capture_request` 的 atlas/capture RGBA 资源样本，所以 current-frame card-capture truth 不再先影响 RT lighting、却在同一 probe encode 里被 irradiance 侧静默丢掉
- 这批 persisted page sample 现在还会在 runtime 内部继续上抬成 voxel authority：`HybridGiVoxelSceneState::synchronize(...)` 会把 resident 且当前不 dirty 的 owner page `capture_sample_rgba` 回灌进匹配 `dominant_card_id` 的 occupied `voxel_cells.radiance_rgb`，所以 clean frame 导出的 `voxel_cells` 不再只剩 tint/direct-light placeholder truth
- `apply_scene_prepare_resources(...)` 在 GPU completion 回灌 `HybridGiSurfaceCacheState` 之后，也会立刻把同一批 persisted page sample 推回当前 `HybridGiVoxelSceneState`；这样下一次 clean-frame `HybridGiScenePrepareFrame` 即使剥掉 `surface_cache_page_contents` 也仍然保有 runtime-owned voxel radiance truth
- dirty resident page 会在 scene sync 时被这层 page-sample overlay 显式排除，避免旧的 persisted sample 在当前 recapture frame 抢先覆盖掉还未完成更新的 voxel authority
- 这条 persisted-page seam 现在还补上了 owner/page 真正分离后的内部合同：`build_scene_prepare_frame.rs` 会按 `owner_card_id` 取 bounds 导出 clean-frame page content，`scene_prepare_surface_cache_owner_rgb(...)` 与 `HybridGiVoxelSceneState` 都改成按 owner card 匹配 persisted page，而 `create_buffers.rs` staging synthetic clean-frame card descriptor 时也改成 `primary_id = owner_card_id`、`secondary_id = page_id`。dirty 排除和 residency 仍然按 `page_id` 工作，因此 owner lookup 与 page lifecycle 不再被混成一条 id

这意味着 Milestone 1 现在第一次有了真正可消费的 runtime-owned persistent surface-cache content layer。它还不是完整的 persistent GPU atlas/page-table residency manager，也还没有把最终屏幕命中的 surface-cache lookup 全部接到 page reuse 上，但 clean-frame scene-prepare/readback、synthetic card-descriptor GPU completion、owner-card resolve fallback、surface-cache spatial fallback、hierarchy irradiance fallback、encoded current-frame card-capture irradiance fallback 与 runtime voxel radiance rehydration 已经不再依赖“本帧必须重新 capture 才有 truth”。

### Runtime-Owned Voxel Cell Residency Contract

这一轮继续把 voxel residency 的权威来源从 renderer-local mesh iteration 往 runtime host 收了一层，但只先收结构性真值，不提前伪装最终 voxel shading authority 已经完成：

- `HybridGiVoxelSceneState` 现在会在 runtime `synchronize(...)` 阶段，按每个 resident clipmap 固定的 `4x4x4` cell grid 生成 `HybridGiPrepareVoxelCell { clipmap_id, cell_index, occupancy_count, dominant_card_id, radiance_rgb }`
- `HybridGiScenePrepareFrame` 现在除了 `card_capture_requests + voxel_clipmaps` 之外，还会继续导出 `voxel_cells`
- runtime 为每个 resident clipmap 固定导出完整 `64` 个 cell entry，而不是只导出 occupied cell；这样 renderer/readback 侧可以保持 deterministic cell ordering
- 这份 payload 现在已经固定了 coarse residency density、dominant contributor ownership，以及 dominant tint + split direct-light seed；但它仍然不编码完整材质/表面 cache/emissive 真值，也还不是最终的软件 voxel lighting authority

这条 cutover 的目标很明确：Milestone 1 先让 `scene representation -> runtime scene prepare` 真正拥有 voxel cell occupancy/count truth，而不是继续把 renderer 里的 `scene_meshes` 重算当长期权威。

### Renderer Scene-Prepare GPU Contract

Milestone 1 这轮又把这份 scene-driven truth 继续推进到了 renderer prepare seam，而不是只停在 runtime host：

- `HybridGiRuntimeState::build_scene_prepare_frame()` 会从 `HybridGiSceneRepresentation` 导出 `HybridGiScenePrepareFrame`
- `submit_frame_extract` 会把这份内部 frame 放进 `ViewportRenderFrame.hybrid_gi_scene_prepare`
- `SceneRendererCore::execute_runtime_prepare_passes(...)` 再把它和旧的 `HybridGiPrepareFrame / HybridGiResolveRuntime` 一起送进 Hybrid GI GPU prepare
- `collect_inputs(...)` 现在除了透传 `card_capture_requests / voxel_clipmaps` 之外，也会把 `voxel_cells` 一起带进 renderer prepare 输入；因此 renderer 当前已经能在不重新扫描 scene mesh 的情况下消费 runtime-owned voxel occupancy/count/ownership/color truth
- renderer prepare 仍然会把 `frame.extract.geometry.meshes` 与 split-light `directional/point/spot` 一并送进 `collect_inputs(...)`，但这条 mesh 输入当前只继续服务 card-capture shading、voxel radiance sample 和 dominant-contributor debug，不再负责长期持有 voxel occupancy/count authority

renderer 端没有再为 cards 和 voxel clipmaps 各开一条独立 storage-buffer，而是显式收束成一条统一的 `scene_prepare_descriptor_buffer`：

- binding `4` 现在固定给只读 scene-prepare descriptor buffer
- 原本的 completed/irradiance/trace-lighting 输出顺延到 bindings `5..8`
- 之所以这样收束，是因为当前机器上的 `wgpu` compute-stage storage-buffer 上限只有 `8`；如果 cards 和 voxels 各占一条独立 buffer，会直接超过 binding limit

`update_completion.wgsl` 也不再只是“把 scene descriptor 绑上去但完全不读”。当前 shader 已经开始真实消费这份统一 descriptor：

- `card_capture_requests` 会按 `card/page/atlas/capture/bounds` 量化后进入 GPU
- `create_buffers.rs` 现在也会在 card descriptor staging 阶段复用同一份 `scene_card_capture_rgba(...)` 真实场景 seed，把 `RGB` 打进 card descriptor 的 `_padding0`，并用 `_padding1` 明确标记“这是显式 packed seed 而不是缺省值”；因此 `RGB = [0, 0, 0]` 的黑色 seed 不会再和“没有 packed seed，只能退回旧逻辑”混为一谈
- `voxel_clipmaps` 会按 `clipmap_id/center/half_extent` 量化后进入 GPU
- `voxel_cells` 现在也会按 `clipmap_id/cell_index/occupancy_count/dominant_card_id/radiance_rgb/cell_center/cell_half_extent` 量化后进入 GPU；`create_buffers.rs` 会把 runtime-owned `radiance_rgb` 打进 unified descriptor 的 `quaternary_id`，并把 `dominant_card_id` 打进 `_padding0`
- `update_completion.wgsl` 对 card-capture descriptor 现在也不再只靠 `card/page/slot/bounds` 的 synthetic 数学推色；当 `_padding1 != 0` 时，它会优先直接解出 `_padding0` 里的 packed card seed，只有旧 fixture 或缺失 packed seed 的 descriptor 才继续退回旧的 synthetic card color
- 这条 real-seed authority 现在也继续贯穿到了当前 frame 的 final resolve：`render/render.rs -> execute_post_process_stack.rs -> execute/run/execute.rs -> write_hybrid_gi_buffers/write.rs -> encode_hybrid_gi_probes/encode.rs` 会把本帧 `HybridGiGpuPendingReadback` 持有的 `scene_prepare_resources` snapshot 只读透传进 `hybrid_gi_hierarchy_rt_lighting.rs`，owner-matched voxel miss fallback 会先按 `capture_slot_id`、再按 `atlas_slot_id` 读取真实 slot sample，而不再直接退回 `scene_prepare_card_capture_request_rgb(...)` 的 synthetic request math
- `update_completion.wgsl` 对 voxel-cell descriptor 不再只用 synthetic color math；当 `quaternary_id` 非零时，它会优先把这份 runtime `radiance_rgb` 当成 cell color authority，只有 authority 缺失时才回退到旧的 synthetic voxel-cell 色彩
- 当 `quaternary_id == 0` 但 `_padding0` 带有非零 `dominant_card_id` 时，`update_completion.wgsl` 现在会先尝试复用同帧 `card_capture_request` 里匹配 `card_id` 的 scene seed。由于 card descriptor 本身已经先吃到真实 packed card seed，这条 owner reuse 路径不再只跟着 `capture_slot_id` 之类的 synthetic layout 信息走，而是能在 fixed-layout 下继续反映 material/base-color/emissive/direct-light 变化；只有找不到匹配 card request 时才退回 owner-based hash fallback
- shader 会对附近 probe 叠加一层 scene-driven radiance boost，所以 near/far scene descriptor 现在会真实改变 GPU readback

在 unified descriptor buffer 之外，renderer prepare 现在还会继续创建一份 per-frame `scene_prepare_resources` scaffold：

- atlas 纹理尺寸由 `atlas_slot_count` 和固定列数直接推导
- capture 纹理会按 `capture_slot_count` 生成 `2D-array` 资源与逐 layer view
- scene-driven `card_capture_requests` 现在会被编码成第一版 scene-driven direct-light seed RGBA，并真实写进 atlas tile 与 capture layer；当前 seed 来源是 matching mesh 的 `tint` 加上当前 frame 的 `directional/point/spot` lights，缺失 matching mesh 时才会回退到 deterministic debug RGBA
- 同一份 `scene_card_capture_rgba(...)` 结果现在不只写进 atlas/capture 纹理，也会同步进入 unified card descriptor；scene-prepare texture path 和 GPU completion descriptor path 因此开始共用同一份 card seed 真值，而不是前者写真实 texel、后者继续靠 slot/id 公式猜色
- 这些纹理、views 和 upload buffers 当前通过 `HybridGiGpuPendingReadback` 保活到 frame 完成，再以 `HybridGiScenePrepareResourcesSnapshot` 形式进入 `HybridGiGpuReadback`
- snapshot 会显式暴露 `occupied_atlas_slots / occupied_capture_slots / atlas_slot_count / capture_slot_count / atlas_texture_extent / capture_texture_extent / capture_layer_count`
- snapshot 现在还会带回 `atlas_slot_rgba_samples / capture_slot_rgba_samples`，用于验证每个 occupied slot/layer 的真实 texel 内容
- snapshot 现在还会带回 `voxel_clipmap_rgba_samples`，用于验证每个 resident clipmap 的最小 radiance seed 样本，而不必只通过 `update_completion.wgsl` 对 probe readback 的间接偏置来判断 voxel scene 有没有活起来；当 runtime 为 clipmap 内的 occupied cells 提供了非零 `radiance_rgb` 时，这个 aggregate clipmap sample 现在也会优先从 runtime `voxel_cells` 按 `occupancy_count` 加权聚合，而不是继续只依赖 renderer-local mesh/material/light path
- snapshot 现在还会带回 `voxel_clipmap_occupancy_masks`，用固定 `4x4x4` clipmap-local occupancy grid 的 `u64` bitmask 去证明 scene mesh 平移时，voxel residency/readback 也会同步变化，而不是只剩颜色样本会变
- snapshot 现在还会带回 `voxel_clipmap_cell_rgba_samples`，把固定 `4x4x4` clipmap-local grid 的每个 cell RGBA 样本都压回 readback；这让 Milestone 1 不只知道 clipmap 是否被激活，还能观察最粗一层 voxel volume content 是否跟着 scene mesh/material/light translation 一起迁移
- snapshot 现在还会带回 `voxel_clipmap_cell_occupancy_counts`，并且其数据源已经完全 cutover 到 runtime-owned `voxel_cells`：renderer 只负责把这份 fixed-grid payload 展开成 per-cell count 与 occupancy mask；当 payload 为空时，occupancy/count 就保持为零，不再回退到旧的 mesh-derived cell count 路径
- snapshot 现在也会在 `voxel_clipmap_cell_rgba_samples / voxel_clipmap_cell_dominant_rgba_samples` 上优先消费 runtime-owned `voxel_cells` 的独立 radiance presence 合同：当 runtime 为某个 occupied cell 提供了 `radiance_present == true` 的 scene authority，scene-prepare readback 会直接把这份颜色权威写回；即使 `radiance_rgb == [0,0,0]` 也会保留为显式黑色 authority，只有 `radiance_present == false` 时才继续退回 renderer-local mesh/material/light voxel debug sample
- snapshot 现在还会带回 `voxel_clipmap_cell_dominant_node_ids`，把同一固定 `4x4x4` grid 下每个 cell 当前由哪个 mesh 主导也压回 readback；当 runtime 为某个 occupied cell 提供了非零 `dominant_card_id` 时，这份 dominant-node readback 现在也会优先消费 runtime voxel payload，而不是继续只从 renderer-local scene meshes 回推 ownership。这让 Milestone 1 可以在重叠 contributor 存在时区分“一个 cell 被多少 mesh 命中”与“最终哪一个 mesh 是 coarse voxel authority”
- snapshot 现在还会带回 `voxel_clipmap_cell_dominant_rgba_samples`，把同一固定 `4x4x4` grid 下每个 cell 当前 dominant contributor 自己的 radiance 颜色也压回 readback；这让 Milestone 1 可以继续区分“整个 coarse voxel cell 聚合后的能量/颜色”与“当前真正主导这个 cell 的 contributor 颜色”，避免 overlapping mesh 只剩 authority id 而没有 authority color truth
- renderer-local `voxel_clipmap_rgba_samples / voxel_clipmap_cell_rgba_samples / voxel_clipmap_cell_dominant_rgba_samples` 现在还共享了一条显式 presence 合同：只要当前 frame 真有 scene mesh 为该 clipmap/cell 贡献样本，RGBA 的 `alpha` 就写成 `255`；完全没有 renderer-side sample 时才写成 `0`。这样当前 frame 的“显式黑色 voxel radiance”终于可以和“没有样本”分开表示，而不会再被压扁成同一个 `[0,0,0]`

换句话说，这个 checkpoint 已经把 runtime-owned `voxel_cells` 从单纯的 `occupancy/count authority` 推到 `occupancy/count + dominant contributor id + dominant tint/direct-light seed authority`，而且不再只停在 resolve/GPU completion 两条链上：`HybridGiVoxelSceneState` 现在会把 scene mesh 的 dominant contributor id 与 `tint + split direct-light seed` 一起量化进每个 cell，resolve miss fallback、GPU completion readback、以及 `scene_prepare` debug/readback 都会优先消费这份 scene-driven 体素真值。最新这层又把 dominant owner 本身进一步连到了同帧 `card_capture_request` scene seed：只要 runtime 没给出 `radiance_present == true` 的 cell radiance，但仍给出 dominant owner，GPU completion 不会再直接压回空间启发式，而是优先复用匹配 card 的 capture seed；而一旦 runtime 明确给出 `radiance_present == true`，即便颜色是 `[0,0,0]`，resolve/GPU/readback 也会把它当成显式黑色 authority 保留下来。clipmap aggregate sample 本身同样已经收回到 runtime cell authority，只要 runtime 给出了 `radiance_present == true` 的 cells，`voxel_clipmap_rgba_samples` 就会按 runtime cell occupancy 加权聚合出 clipmap 颜色，而不是继续只依赖 renderer-local voxel debug 着色。更深一层的 material/emissive/direct-light shading authority 仍然刻意留在 `voxel_clipmap_debug.rs` 的 renderer-local mesh/material/light path，作为下一层更深的 authority 收束点。

这一步仍然主要是 Milestone 1 的 seam 验证，不代表 surface cache 或 voxel fallback 已经达到最终 lighting 质量；但它已经把 “scene representation -> runtime frame -> renderer descriptor buffer + per-frame atlas scaffold -> shader consumption / readback observability” 这条链路真正闭合起来了。

### First Resolve-Side Software Voxel Fallback

在此基础上，resolve 侧现在也不再只有 probe-style trace continuation。`hybrid_gi_hierarchy_rt_lighting(...)` 已经开始在“当前帧没有 scheduled trace region 命中”时读取 `ViewportRenderFrame.hybrid_gi_scene_prepare`：

- runtime exact / ancestor / descendant RT-lighting continuation 仍然优先，保持原有 runtime history 语义不变
- 只有当前 trace 路径给不出有效 RT-lighting 时，才会转到 `scene_prepare` 的 voxel fallback
- fallback 当前先使用 `voxel_clipmaps + voxel_cells` 的 fixed-grid 空间真值来重建 cell center / cell extent，并对附近 probe 给出第一版 cell-level 软件 voxel RT-lighting
- `voxel_cells` 现在不再只是 occupancy/count；runtime scene representation 会把每个 cell 的 dominant mesh tint 量化成 per-cell `radiance_rgb`，resolve 在有这份 scene-driven 色彩权威时优先使用它
- 当 `radiance_rgb` 缺失但 `dominant_card_id` 有效时，resolve 现在也会先尝试复用 `scene_prepare.card_capture_requests` 里匹配 `card_id` 的 scene seed；只有 owner 找不到匹配 card request 时，才继续退回 clipmap-local 空间启发式
- 如果当前 frame 只有 clipmap descriptor 而没有有效 `voxel_cells`，resolve 也会退回 clipmap-level coarse fallback，而不是直接回到纯黑
- 当前的 runtime voxel authority 仍然不是完整的材质/表面 cache 采样；当 `radiance_rgb` 缺失时，resolve 现在会先走 matched card-capture seed，再退回 clipmap-local 空间启发式来避免 miss-path 直接变黑
- 这一层现在又往前收了一步：只要当前帧 `scene_prepare_resources` 里已经有 renderer 侧生成的 voxel sample，resolve 在 runtime `radiance_present == false` 时会优先尝试 `voxel_clipmap_cell_dominant_rgba_samples / voxel_clipmap_cell_rgba_samples`，而 coarse clipmap fallback 也会优先尝试 `voxel_clipmap_rgba_samples`，不再一上来就掉回纯空间启发式。renderer-side 资源路径继续按 sample `alpha > 0` 判断 authority presence，而 runtime-owned `voxel_cells` 则新增了独立 `radiance_present` 位；因此无论 authority 来自当前 frame 的 renderer sample 还是 runtime voxel payload，显式黑色 GI 样本都不会再和“没有 authority”混成同一个 `[0,0,0]`。

Milestone 3 的 productization 现在又沿着这条 seam 往前推了一小步：只要 `ViewportRenderFrame.hybrid_gi_scene_prepare` 已经在场，post-process 就不再让 authored `trace_region` 重新作为主渲染结果的直接输入。`count_scheduled_trace_regions(...)` 与 `encode_hybrid_gi_trace_regions(...)` 会在这种 scene-driven frame 上返回 `0`，而 `hybrid_gi_hierarchy_rt_lighting(...)` 也不再把 authored trace-region RT tint 当成 scene-driven frame 的最终兜底；它会保留 runtime scene truth，或者直接回到 scene-prepare 的 voxel/surface fallback。这样一来，旧的 `trace_region boost + tint` 仍可留在没有 `scene_prepare` 的 fixture-only 迁移路径里做兼容验证，但不会在 scene-driven frame 上重新夺回主 authority。

因此，当前仓库已经不再是“trace miss 就只能回 probe-only continuation 或纯黑”。即便还没有正式的 screen-trace 命中链，这条 resolve-side software voxel fallback 已经把 milestone 2 里最核心的 miss-path 语义先打通了一版。

### Stats And Readback Surface

Milestone 1 的验收要求之一是 debug/readback。当前仓库没有把 scene/surface/voxel 内部结构直接上抬成 façade DTO，而是沿现有 `HybridGiRuntimeSnapshot -> SubmissionRecordUpdate -> RenderStats` 主链，把这些 scene-driven计数暴露给外部：

- `last_hybrid_gi_scene_card_count`
- `last_hybrid_gi_surface_cache_resident_page_count`
- `last_hybrid_gi_surface_cache_dirty_page_count`
- `last_hybrid_gi_surface_cache_feedback_card_count`
- `last_hybrid_gi_surface_cache_capture_slot_count`
- `last_hybrid_gi_surface_cache_invalidated_page_count`
- `last_hybrid_gi_voxel_resident_clipmap_count`
- `last_hybrid_gi_voxel_dirty_clipmap_count`
- `last_hybrid_gi_voxel_invalidated_clipmap_count`

这里的 `surface_cache_capture_slot_count` 现在语义上等价于“当前待执行的 card capture request 数量”，因为统计链已经改为从 `HybridGiSceneRepresentation::card_capture_request_count()` 取值，而不再只是盲读 surface-cache dirty slot 容器长度。

### Scene-Driven Card Registration And Dirty Scope

当前 card registration 已经开始直接从通用 scene extract 派生：

- 每个 `RenderMeshSnapshot.node_id` 会成为一张内部 card 的初始 authority
- runtime host 不再在每帧 `register_extract(...)` 时重建整个 `scene_representation`
- public Hybrid GI settings 更新只会刷新 settings/budget/debug 侧语义
- 真实 meshes/lights 则通过独立 scene-sync 步骤更新 cards、surface cache 和 voxel scene

这一点很关键，因为 milestone 1 的目标不是“把一份新的 settings DTO 塞进 runtime”，而是让 scene representation 真正开始以通用 scene extract 为真源。

### Mesh / Material / Light Change Invalidation

当前 scene sync 已经具备第一版脏化粒度：

- mesh/card 首次出现时，对应 resident page 会被标记为 dirty capture
- mesh 保持同一个 `node_id`，但 transform / model / material / tint / render-layer 等 snapshot 内容变化时，只会把对应 card/page 重新标 dirty
- directional / point / spot lights 的 scene snapshot 变化时，当前 resident pages 会整体重标 dirty，表示 direct-light seed 需要重新 capture
- voxel clipmaps 则会在 card 集合或光照集合变化时整组重标 dirty

这还不是最终“只失效空间受影响 clipmap brick”的细化实现，但它已经把 milestone 1 需要的语义固定下来：scene change 不再等价于 runtime host 全量重建，也不再是 completely stateless。

### Runtime Continuation Resolve Now Blends Exact, Ancestor, And Descendant Lineage

当前 scene-representation 虽然已经开始 scene-driven 化，但旧 probe-style runtime continuation 仍然参与最终 resolve，所以 lineage 组合规则必须稳定：

- probe 自身的 exact runtime hierarchy entry 不能再直接遮蔽 descendant continuation
- ancestor gather、exact entry、requested descendant continuation 现在会在同一轮中统一加权混合
- RGB lineage source 只在最终输出阶段做一次 support clamp，避免中途先 blend 再 clamp 导致 descendant 贡献被提前压扁
- resolve-weight lineage 也遵循同一条规则，保证 descendant continuation 不会因为 parent exact weight 非零就完全失效
- 这条 seam 现在又补上了第一个 runtime-host scene-driven hole-fill：当 `scheduled_trace_region_ids` 为空、probe-style hierarchy irradiance 自身也没有 continuation 时，`build_resolve_runtime.rs` 会直接从当前 runtime-owned persisted `surface_cache_page_contents` 合成 exact `hierarchy_irradiance`
- `HybridGiResolveRuntime` 现在会给这类 exact irradiance 打上 “already scene-driven” metadata，`hybrid_gi_hierarchy_irradiance.rs` 会据此跳过同一帧对同一份 `scene_prepare` page truth 的二次混合
- 这条 “exact runtime already includes current scene truth” metadata 现在也不再停在 post-process resolve：`runtime_trace_source.rs` 和 `runtime_irradiance_source()` 会把 runtime source 扩成 `(support_q, packed_rgb, includes_scene_truth)`，只对 exact runtime hierarchy entry 暴露 scene-truth bit，ancestor fallback 仍保持普通 continuation 语义
- `pending_probe_inputs.rs` 与 `resident_probe_inputs.rs` 现在会在 `scheduled_trace_region_ids` 为空时分别预计算 `skip_scene_prepare_for_trace_q` 与 `skip_scene_prepare_for_irradiance_q`，这样 trace-lighting 和 irradiance 两条 GPU prepare 路径可以独立决定是否跳过同帧 `scene_prepare` 重混，而不是被迫共用一个粗粒度“全跳/全不跳”开关
- `update_completion.wgsl` 现在也按这两个 skip bit 分开计算 traced contribution：trace-lighting continuation 和 irradiance continuation 各自拥有独立的 scene-prepare reblend 决策，因此 runtime exact RT 已经 scene-driven 时可以只让 RT 路径跳过 page/voxel truth 重混，而 irradiance 路径仍可继续消费 scene-prepare，反之亦然
- 这样 GPU prepare / pending-probe readback 终于和 helper-level exact-runtime contract 对齐：空 trace schedule 下，已经 scene-driven 的 runtime exact source 不会把同一份 persisted page / voxel truth 在 GPU encode 里又混第二次

这一步把 renderer resolve 和 runtime host 的 continuation 语义重新对齐，不再出现“parent exact entry 一旦非零就把 child continuation 硬切掉”的结果。

### Voxel Clipmap Budgeting

`HybridGiVoxelSceneState` 当前也开始跟随 scene card 集合变化维护 clipmap residency：

- 有 active cards 时，`voxel_budget` 决定 resident clipmap 数
- resident clipmap 会从当前 card bounds 计算 deterministic descriptor：`center` 来自 scene bounds 中心，`half_extent` 从 scene 最大跨度向上取整后按 clipmap 层级倍增
- card 集合变化会把当前 resident clipmaps 全部标记为 dirty
- 没有 active cards 时，resident clipmaps 会清空
- runtime host 级 scene registration 现在也能在测试里直接读回这些 descriptor 与 invalidation 结果，确保 scene extract -> runtime host -> voxel scene 这条链路是闭合的
- runtime host 现在还会随 resident clipmap 一起导出固定 `4x4x4` 的 `voxel_cells` occupancy/count/ownership/color payload；renderer scene-prepare 会把这份 payload 压成 `u64` occupancy mask、per-cell count/dominant-node readback、以及 unified descriptor buffer 的 owner/color authority，不再自行从 `scene_meshes` 回算这些长期真值

这仍然是 milestone 1 的 skeleton，不是最终软件 voxelization 结果；但它已经把“scene change 会驱动 voxel fallback 更新”这条状态语义固定下来。

## Current Verification

这轮已经明确通过的定向验证包括：

- split-light scene extract world roundtrip
- scene asset TOML roundtrip for point / spot light
- frame extract split-light roundtrip
- `RenderHybridGiExtract` 默认 public settings 语义
- `HybridGiInputSet` 的 Deferred / Forward+ 完整性
- `HybridGiSceneRepresentation::from_extract(...)` 对 public settings 和 internal fixture bridge 的分离
- runtime host scene-card registration from real mesh extract
- runtime host selective dirtying for material changes and full resident relight for scene-light changes
- deterministic surface-cache page-table slot reuse and card-capture atlas slot reuse
- resident page capture-slot reservation across clean-to-dirty transitions
- scene-driven card-capture request descriptors carrying `card/page/atlas/capture/bounds` truth
- runtime-host test accessors exposing those card-capture requests end-to-end
- runtime-host persistent surface-cache page samples surviving clean frames and invalidation
- runtime-host clean-frame `HybridGiScenePrepareFrame` re-export of persisted `surface_cache_page_contents`
- render-framework GPU completion feeding `scene_prepare_resources` back into runtime surface-cache state instead of dropping them at the submission seam
- `RenderStats` readback for scene-card / surface-cache / voxel-scene Milestone 1 counters
- unified `scene_prepare_descriptor_buffer` staging for card-capture requests, voxel clipmaps, and runtime-owned voxel cells
- fixed-layout GPU owner fallback when only runtime `dominant_card_id` changes and `radiance_rgb` stays zero
- fixed-layout GPU owner fallback reusing matched scene card-capture seed when only that card seed changes
- `update_completion.wgsl` consuming near-field scene descriptors so renderer readback changes when card / clipmap / voxel-cell scene-prepare data moves
- resolve helper lineage blending for exact parent entries plus descendant continuation
- resolve-side software voxel fallback from `hybrid_gi_scene_prepare` when no current trace support exists
- coarse clipmap-level resolve fallback surviving even when runtime omits voxel-cell payload
- resolve-side voxel fallback now preferring nonzero current-frame `scene_prepare_resources` voxel cell / clipmap samples before falling back to matched owner-card seed or pure spatial heuristic
- renderer scene-prepare atlas/capture readback now reusing persisted surface-cache page samples even when `card_capture_requests` is empty
- resolve-side owner-card fallback now reusing persisted clean-frame `surface_cache_page_contents` when no current card-capture request exists
- render-level final GI resolve now changing with persisted clean-frame surface-cache page samples even when runtime voxel layout and owner stay fixed
- render-level regression coverage for irradiance / RT lighting / resolve-weight descendant continuation
- targeted `page_table_and_capture_slots` and `reuses_surface_cache_slots_after_invalidation` coverage
- targeted `card_capture_requests` coverage
- full `hybrid_gi_scene_representation` coverage including scene-bounds-driven voxel clipmap descriptors
- runtime-host coverage for scene clipmap descriptor construction and scene-clear invalidation
- targeted renderer seam coverage for:
  - scene-prepare quantization
  - collect-inputs scene-prepare passthrough
  - runtime scene-prepare voxel-cell export with deterministic `64`-cell occupancy payload per resident clipmap
  - scene-prepare atlas/capture resource snapshot readback
  - deterministic atlas/capture texel sample readback for occupied slots
  - atlas/capture samples responding to mesh tint plus directional-light changes
  - atlas/capture samples responding to point-light and spot-light changes
  - atlas/capture samples responding to material base-color differences
  - atlas/capture samples responding to material emissive differences without direct lights
  - voxel clipmap samples responding to material emissive differences without direct lights
  - voxel clipmap occupancy masks reacting to scene-mesh translation across clipmap cells
  - voxel clipmap cell radiance samples following scene-mesh translation across clipmap cells
  - voxel clipmap cell occupancy counts accumulating overlapping meshes inside the same coarse voxel cell
  - voxel clipmap occupancy/count readback honoring runtime-owned `voxel_cells` payload even when renderer-local scene meshes are absent
  - voxel clipmap occupancy/count readback staying zero when runtime omits `voxel_cells`, even if renderer-local scene meshes are present
  - voxel clipmap dominant node ids preferring the brighter overlapping contributor inside the same coarse voxel cell
  - voxel clipmap dominant RGBA samples preserving the brighter overlapping contributor separately from the aggregate coarse-cell sample
  - GPU readback reacting to near/far card-capture descriptors
  - GPU readback reacting to near/far voxel clipmaps
  - GPU readback reacting to near/far runtime `voxel_cells` while clipmap truth stays fixed
  - final GI resolve reacting to `scene_prepare` voxel-cell fallback when no trace schedule exists
- final GI resolve reacting to `scene_prepare` voxel-clipmap fallback even when runtime voxel cells are absent
- final GI resolve reacting to runtime scene voxel tint authority even when voxel layout stays fixed
- final GI resolve reacting to matched scene card-capture seed when runtime voxel layout and owner stay fixed but per-cell radiance is absent
- clean-frame runtime voxel radiance rehydrating from persisted surface-cache page samples instead of keeping the old tint/direct-light placeholder
- final GI resolve reacting to clean-frame persisted page samples even after `surface_cache_page_contents` are removed from renderer input, proving runtime voxel radiance itself now carries that authority
- GPU completion reacting to clean-frame persisted surface-cache page descriptors even when there are no dirty card-capture requests and no runtime voxel radiance fallback
- absent clean-frame persisted page samples (`atlas/capture alpha = 0`) matching the no-page GPU baseline instead of fabricating false black descriptor authority
- atlas-only persisted page samples reusing `atlas_sample_rgba` as descriptor seed when `capture_sample_rgba` is absent
- atlas-only persisted page samples occupying only atlas-side scene-prepare resources, while capture-only persisted page samples occupy only capture-side resources
- runtime surface-cache reapplication preserving atlas-only or capture-only persisted pages across clean frames instead of dropping them until both sides exist
- resolve-side current `card_capture_request` resource fallback treating `alpha = 0` capture/atlas samples as absent, reusing atlas truth when capture is absent, and falling through to synthesized request seed when both sides are absent
- final GI resolve reacting to persisted surface-cache page samples even when runtime voxel clipmaps/cells are absent
- hierarchy irradiance reacting to atlas-only or capture-preferred persisted surface-cache page samples when runtime irradiance and ancestor prepare irradiance are absent, instead of collapsing back to `[0, 0, 0, 0]`
- encoded probe hierarchy irradiance reacting to atlas-only or capture-preferred current-frame `scene_prepare_resources` card-capture samples, instead of collapsing back to the same synthesized request RGB on both frames
- `GlobalIllumination` temporal signature now also mixes the local surface-cache fallback RGB/support from current-frame `scene_prepare` truth, so a warm-to-cool page-sample flip no longer keeps reusing stale GI history just because the probe lineage itself stayed unchanged
- exact runtime RT continuation now also blends current `scene_prepare` surface-cache/voxel fallback when `scheduled_trace_region_ids` is empty, so stale `probe_rt_lighting_rgb` no longer flattens warm/cool clean-frame page-truth flips back to the same runtime-only color
- exact runtime irradiance continuation now also blends current `scene_prepare` surface-cache fallback when `scheduled_trace_region_ids` is empty, so stale `hierarchy_irradiance` no longer flattens warm/cool clean-frame page-truth flips back to the same runtime-only color
- runtime-host exact `hierarchy_irradiance` now also has its own scene-driven hole-fill path when `scheduled_trace_region_ids` is empty and probe lineage itself offers no irradiance continuation, so nearby persisted `surface_cache_page_contents` no longer collapse back to `[0, 0, 0, 0]` before the frame even reaches renderer-side exact-runtime blending
- renderer-side exact irradiance now honors runtime metadata that marks those entries as already scene-driven, so the same persisted page truth is not blended twice in one frame
- runtime-host exact `hierarchy_rt_lighting` now also has its own scene-driven hole-fill path when `scheduled_trace_region_ids` is empty and probe lineage itself offers no RT continuation, so nearby runtime-owned voxel cells and persisted surface-cache truth no longer collapse back to empty exact RT inputs before renderer-side blending
- `HybridGiResolveRuntime` now carries matching scene-driven RT metadata, and `hybrid_gi_hierarchy_rt_lighting.rs` consumes that bit to skip reblending the same current-frame voxel/page truth a second time when runtime exact RT is already scene-aware
- pending-probe GPU irradiance encode now has explicit coverage for the same empty-trace exact-runtime seam, so higher-level probe readback stays aligned with the helper-level scene-driven irradiance contract
- pending-probe GPU trace-lighting encode now honors the same scene-driven exact-runtime metadata instead of always reblending current-frame `scene_prepare` truth, so warm/cool runtime exact RT no longer collapses back toward the same duplicated page color on empty-trace frames
- resident/pending GPU prepare inputs now carry separate `skip_scene_prepare_for_trace_q` and `skip_scene_prepare_for_irradiance_q` flags, which keeps trace-lighting and irradiance encode behavior symmetrical with the distinct runtime exact RT / exact irradiance scene-truth bits
- `GlobalIllumination` temporal signature now also mixes scene-driven exact runtime hierarchy truth across exact, inherited, and descendant lineage sources, so a warm-to-cool runtime-only flip no longer keeps reusing stale GI history when current-frame `scene_prepare` page truth is absent or intentionally stripped from the renderer input
- `encode_hybrid_gi_probes(...)` now also normalizes current scene-driven surface-cache/runtime support into per-probe temporal confidence, accumulates reinforcing scene-driven sources instead of collapsing to the strongest source only, and `post_process.wgsl` uses that continuous value to give scene-truth sources extra temporal-confidence headroom without flattening pure continuation back into an almost weight-insensitive history blend when the temporal signature remains stable
- `hybrid_gi_temporal_signature.rs` now also applies explicit scene-truth provenance scales on top of that support weighting: exact runtime truth stays at `1.0`, inherited runtime truth is reduced to `0.85`, descendant runtime truth is reduced to `0.7`, and current surface-cache/page proxy truth is capped at `0.85`, so a similarly-supported capture/page fallback no longer ties exact runtime truth in temporal history confidence
- `scene_prepare_surface_cache_samples.rs` now also feeds a richer proxy-quality signal into that same confidence path: current capture resources stay most trusted, atlas-only resource samples are slightly discounted, persisted capture/atlas samples are discounted further, and synthetic request RGB is the lowest-confidence proxy, so placeholder request bounds no longer reuse GI history as aggressively as real card-capture truth at the same spatial support
- `HybridGiResolveRuntime` now also carries per-probe scene-truth quality metadata beside the existing scene-driven flags, and `hybrid_gi_temporal_signature.rs` multiplies runtime temporal confidence by both lineage provenance and runtime source quality instead of treating voxel radiance, capture-backed surface cache, atlas-backed surface cache, and purely spatial voxel fallback as equally trustworthy whenever their support is identical
- `build_resolve_runtime.rs` now derives that runtime quality from the actual scene-owned miss source: voxel-radiance exact truth stays at full quality, capture-backed surface cache is discounted slightly, atlas-only surface cache is discounted more, and purely spatial voxel fallback is the weakest runtime scene-truth source. The new runtime/history regressions keep that ordering visible in both resolve-runtime metadata and final `GlobalIllumination` history reuse
- `HybridGiResolveRuntime` now also carries per-probe scene-truth freshness metadata beside quality, and `hybrid_gi_temporal_signature.rs` multiplies runtime temporal confidence by dirty/invalidation-driven freshness so equally supported scene-truth sources stop reusing `GlobalIllumination` history as if dirty pages or dirty clipmaps were already stable
- `build_resolve_runtime.rs` now derives that freshness from runtime-owned scene state itself: dirty surface-cache pages, dirty voxel clipmaps, and coarse surface/voxel invalidation counts each reduce freshness, while owner-matched surface reuse and voxel miss fallback propagate the tighter freshness of the page/clipmap authority they actually consumed instead of claiming clean-frame confidence they do not have
- `hybrid_gi_temporal_signature.rs` now also folds blended runtime scene-truth quality + freshness into dedicated irradiance/RT validity seeds inside the temporal signature itself, so a clean-to-dirty surface-cache or voxel transition now resets `GlobalIllumination` history even when runtime RGB/support remain bit-for-bit identical
- `HybridGiResolveRuntime` now also carries per-probe scene-truth revision/change-serial metadata for both exact irradiance and exact RT entries, and `hybrid_gi_temporal_signature.rs` folds exact/inherited/descendant revision hashes into dedicated irradiance/RT revision seeds. `HybridGiVoxelSceneState::synchronize(...)` now bumps that revision on semantic `scene_changed` as well as on structural clipmap/cell diffs, so light/topology/material changes still invalidate `GlobalIllumination` history even after fixed-grid voxel payload stabilizes back to the same RGB/support/freshness
- exact runtime RT revision now also follows the actual authority source instead of always pretending to be voxel-owned: voxel-backed exact RT keeps `voxel_scene.scene_revision()`, while the surface-cache-backed RT hole-fill used by empty-voxel fixtures returns `surface_cache.scene_revision()`. Exact runtime irradiance scene truth still remains surface-cache-owned, so voxel-only fixtures intentionally keep exact irradiance revision absent while exact RT revision drives the temporal reset
- `scene_prepare_surface_cache_samples.rs` keeping request/page/owner/spatial fallback semantics converged across RT lighting and irradiance instead of drifting into duplicate fallback logic
- continuation-only history reuse now keeps full resolve-weight sensitivity again: `HYBRID_GI_HISTORY_CONTINUATION_CONFIDENCE_SCALE` is back at `1.0`, while scene-truth sources still gain extra confidence through `HYBRID_GI_HISTORY_SCENE_TRUTH_CONFIDENCE_RANGE`, so a strong `hierarchy_resolve_weight` once again preserves visibly more second-frame GI history than a weak one under identical screen support instead of collapsing both paths toward the same flat temporal blend
- scene-driven final composite now also strips authored resident-probe irradiance tint from the post-process probe buffer itself, so once `hybrid_gi_scene_prepare` is present the final GI color comes from runtime surface-cache / voxel truth plus continuation metadata instead of being recolored by legacy `resident_probes[].irradiance_rgb`
- scene-driven final composite now also replaces authored probe `position/radius` as its spatial carrier: post-process probe screen data is re-derived from aggregated `scene_prepare` surface bounds, falling back to voxel-clipmap bounds only when no card/page bounds exist, so moving authored probe screen position no longer shifts scene-driven GI while moving scene-owned page bounds still does
- scene-driven final composite now also drops unmatched `prepare.resident_probes` before post-process probe encoding, so compatibility-only probe slots that have no authored extract source no longer enter the probe buffer or inflate `probe_count` and dim the final GI through legacy container semantics
- `hybrid_gi_temporal_signature.rs` now also switches to a neutral scene-truth seed whenever current-frame surface-cache or runtime scene truth is present, so scene-driven exact/runtime-backed GI history no longer resets purely because legacy `probe_id/parent_probe_id` changed while the underlying scene truth stayed identical
- `hybrid_gi_hierarchy_resolve_weight.rs` now also neutralizes authored hierarchy fallback on scene-driven frames when no runtime resolve-weight authority exists, so current `scene_prepare` truth no longer gets reweighted by legacy `parent/child/budget` lineage rules before final composite or temporal-confidence weighting
- `hybrid_gi_hierarchy_irradiance.rs` and `hybrid_gi_hierarchy_rt_lighting.rs` now also short-circuit inherited/descendant lineage gathering when the current probe already carries exact runtime scene truth, so descendant lineage tint no longer perturbs final scene-driven irradiance or RT composite once exact authority exists
- `hybrid_gi_temporal_signature.rs` now applies the same short-circuit to exact scene-truth temporal signature, confidence, and revision inputs, so descendant probe-id churn no longer resets `GlobalIllumination` history reuse while the underlying exact runtime truth stays stable
- `runtime_parent_chain.rs` now also stops folding authored ancestor/descendant probe ids into scene-truth lineage revision accumulation, so parent probes that only inherit scene truth through non-exact descendant runtime lineage no longer drop `GlobalIllumination` history on pure descendant probe-id churn when runtime support and revision stay fixed
- `hybrid_gi_temporal_signature.rs` now also gathers non-exact descendant scene truth without authored depth falloff for temporal signature/confidence/revision, so inserting an intermediate authored descendant node no longer changes `GlobalIllumination` history reuse while the same leaf runtime scene truth stays fixed
- targeted Hybrid GI runtime/encode/history/render suites remain green, and a fresh `cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-history` now also passes on the same target dir
- fresh `target/codex-hybrid-gi-trace-demotion` validation now also keeps the neighboring continuation-only temporal regression green: `hybrid_gi_resolve_preserves_more_history_when_hierarchy_resolve_weight_is_stronger`, the full `graphics::tests::hybrid_gi_resolve_render` module, and `cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-trace-demotion` all pass together
- the same `target/codex-hybrid-gi-trace-demotion` lane now also keeps `hybrid_gi_resolve_scene_driven_frame_ignores_prepare_probe_irradiance_tint_changes` green beside the trace-region demotion regressions, so scene-driven frames no longer keep a second authored tint authority in the final composite after trace-region direct influence was removed
- the same validation lane now also keeps both `hybrid_gi_resolve_scene_driven_frame_ignores_prepare_probe_screen_position_changes` and `hybrid_gi_resolve_scene_driven_frame_localizes_from_scene_prepare_bounds_instead_of_probe_position` green, so scene-driven final composite is no longer spatially anchored to authored probe coordinates but still responds to moved scene-owned page bounds
- the same validation lane now also keeps `hybrid_gi_resolve_scene_driven_frame_ignores_unmatched_prepare_probe_slots` green, so adding a compatibility-only resident probe slot with no authored extract source no longer changes scene-driven final GI energy or spatial distribution through legacy probe-container normalization
- the same validation lane now also keeps `graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_scene_driven_exact_runtime_truth_keeps_history_when_only_probe_identity_changes` and the full `graphics::tests::hybrid_gi_resolve_history` suite green, so scene-driven exact runtime truth no longer loses `GlobalIllumination` history reuse on a pure legacy probe-id transition
- the same validation lane now also keeps `graphics::scene::scene_renderer::post_process::resources::execute_post_process::encode_hybrid_gi_probes::hybrid_gi_hierarchy_resolve_weight::tests::scene_driven_frame_uses_neutral_resolve_weight_without_runtime_authority` and `hybrid_gi_resolve_scene_driven_frame_ignores_authored_parent_child_links` green, so scene-driven frames no longer let authored hierarchy fallback reweight current GI when runtime resolve-weight truth is absent
- the same validation lane now also keeps `scene_driven_exact_runtime_irradiance_ignores_descendant_lineage_tint`, `scene_driven_exact_runtime_rt_lighting_ignores_descendant_lineage_tint`, and `hybrid_gi_resolve_scene_driven_exact_runtime_truth_keeps_history_when_only_descendant_identity_changes` green, while refreshed `encode_hybrid_gi_probes` / `graphics::tests::hybrid_gi_resolve_history` / `graphics::tests::hybrid_gi_resolve_render` runs on `target/codex-hybrid-gi-trace-demotion` still pass together, so exact scene-driven runtime now remains authoritative over descendant lineage tint or descendant-id churn instead of treating that lineage as a second color/temporal truth source
- the same validation lane now also keeps `graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_history_when_only_descendant_identity_changes` green, so non-exact descendant-runtime scene truth no longer resets `GlobalIllumination` history just because the authored descendant probe id changed while runtime scene truth itself stayed fixed
- the same validation lane now also keeps `graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_history_when_only_descendant_depth_changes` green, so non-exact descendant-runtime scene truth no longer resets `GlobalIllumination` history just because an intermediate authored descendant node was inserted while the same leaf runtime scene truth stayed fixed
- the same validation lane now also keeps `graphics::tests::hybrid_gi_resolve_history::hybrid_gi_resolve_scene_driven_inherited_runtime_truth_keeps_history_when_only_ancestor_depth_changes`, `graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_scene_driven_inherited_runtime_truth_keeps_current_gi_when_only_ancestor_depth_changes`, and `graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_scene_driven_inherited_runtime_truth_keeps_scene_prepare_mix_when_only_ancestor_depth_changes` green, so non-exact inherited-runtime scene truth no longer resets `GlobalIllumination` history or changes current GI intensity / scene-prepare mix just because an intermediate authored ancestor node was inserted while the same inherited runtime scene truth stayed fixed; the refreshed `target/codex-hybrid-gi-trace-demotion` lane now passes with `graphics::tests::hybrid_gi_resolve_history` 14 passed, `encode_hybrid_gi_probes` 31 passed, `graphics::tests::hybrid_gi_resolve_render` 55 passed, and `cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-trace-demotion`
- the same validation lane now also keeps `graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_scene_driven_inherited_runtime_truth_ignores_reachable_continuation_weight_from_inserted_ancestor` green, and `hybrid_gi_hierarchy_resolve_weight.rs` now forces scene-driven frames back to neutral `1.0` unless the current probe itself carries exact scene-truth resolve-weight authority, so a continuation-only hierarchy-resolve-weight entry that only becomes reachable after inserting an authored ancestor no longer changes current GI intensity while the inherited runtime scene truth itself stayed fixed; the refreshed `target/codex-hybrid-gi-trace-demotion` lane now passes with `graphics::tests::hybrid_gi_resolve_history` 14 passed, `encode_hybrid_gi_probes` 31 passed, `graphics::tests::hybrid_gi_resolve_render` 56 passed, and `cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-trace-demotion`
- the same validation lane now also keeps `graphics::tests::hybrid_gi_resolve_render::hybrid_gi_resolve_scene_driven_inherited_runtime_truth_ignores_reachable_continuation_rgb_from_inserted_ancestor` green, and both `hybrid_gi_hierarchy_irradiance.rs` and `hybrid_gi_hierarchy_rt_lighting.rs` now split runtime RGB lineage into scene-truth versus continuation branches before final scene-driven selection, so a continuation-only inherited RGB entry that only becomes reachable after inserting an authored ancestor no longer perturbs current GI color while the inherited runtime scene truth itself stayed fixed; the refreshed `target/codex-hybrid-gi-trace-demotion` lane now passes with `graphics::tests::hybrid_gi_resolve_history` 14 passed, `encode_hybrid_gi_probes` 31 passed, `graphics::tests::hybrid_gi_resolve_render` 57 passed, and `cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-trace-demotion`
- the same validation lane now also keeps `encode_hybrid_gi_probes_ignores_surface_cache_proxy_signature_when_exact_runtime_scene_truth_exists` and `encode_hybrid_gi_probes_ignores_surface_cache_proxy_confidence_when_exact_runtime_scene_truth_exists` green, and `hybrid_gi_temporal_signature.rs` now only lets `scene_prepare` surface-cache proxy truth participate in temporal signature/confidence when that proxy actually contributes to the current irradiance path. Once exact runtime scene truth already owns current irradiance, changing only the non-authoritative proxy seed no longer perturbs temporal history identity or confidence. The refreshed `target/codex-hybrid-gi-trace-demotion` lane now passes with `encode_hybrid_gi_probes` 33 passed, `graphics::tests::hybrid_gi_resolve_history` 14 passed, `graphics::tests::hybrid_gi_resolve_render` 57 passed, and `cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-trace-demotion`
- the same temporal encode seam is now closed for non-exact scene-driven runtime lineage too: inherited and descendant runtime scene truth no longer let a non-authoritative `scene_prepare` surface-cache proxy perturb `GlobalIllumination` temporal signature or confidence once current irradiance is already sourced from scene-driven lineage truth. `hybrid_gi_temporal_signature.rs` now gates proxy participation on whether current irradiance actually falls back to surface-cache proxy data, and the refreshed `target/codex-hybrid-gi-trace-demotion` lane keeps `encode_hybrid_gi_probes_ignores_surface_cache_proxy_signature_when_lineage_runtime_scene_truth_exists`, `encode_hybrid_gi_probes_ignores_surface_cache_proxy_confidence_when_lineage_runtime_scene_truth_exists`, the full `encode_hybrid_gi_probes` suite, `graphics::tests::hybrid_gi_resolve_history`, `graphics::tests::hybrid_gi_resolve_render`, and `cargo check -p zircon_runtime --locked --lib --target-dir target/codex-hybrid-gi-trace-demotion` green together with `encode_hybrid_gi_probes` at 35 passed
- end-to-end scene-driven lineage coverage now also locks the same current-page contract at the full renderer/history level instead of only in helper-level encode/composite tests: `hybrid_gi_resolve_scene_driven_inherited_runtime_truth_ignores_scene_prepare_surface_cache_tint` proves inherited runtime scene truth keeps current final composite materially stable when only the current `surface_cache_page_contents` tint changes, while `hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_history_when_only_scene_prepare_surface_cache_tint_changes` proves descendant runtime scene truth keeps `GlobalIllumination` history reuse alive under the same current-page tint churn once the fixture carries full descendant RT-plus-irradiance scene-truth authority. The refreshed `target/codex-hybrid-gi-trace-demotion` lane now keeps `graphics::tests::hybrid_gi_resolve_history` at 15 passed and `graphics::tests::hybrid_gi_resolve_render` at 58 passed
- targeted validation for this temporal validity checkpoint now also includes `hybrid_gi_resolve_rejects_history_when_surface_cache_scene_truth_freshness_changes_without_rgb_change`, `hybrid_gi_resolve_rejects_history_when_voxel_scene_truth_freshness_changes_without_rgb_change`, the full `graphics::tests::hybrid_gi_resolve_history`, `graphics::tests::hybrid_gi_runtime`, `graphics::tests::hybrid_gi_resolve_surface_cache`, and `encode_hybrid_gi_probes` suites, plus the descendant resolve regression groups `hybrid_gi_resolve_uses_descendant_scene_driven_runtime*`, `hybrid_gi_resolve_gathers_requested_descendant_runtime*`, and `hybrid_gi_resolve_blends_nonzero_exact*`

当前这轮 acceptance 仍然以 Hybrid GI 自身的 targeted evidence 为主：`hybrid_gi_scene_prepare_requires_runtime_voxel_cells_for_occupancy_and_count_truth` 已经证明空 `voxel_cells` 不会再触发 renderer fallback，`hybrid_gi_scene_prepare_uses_runtime_voxel_cell_payload_without_scene_meshes` 又把这条 contract 向前推进到完整 color-and-ownership truth，证明即便 renderer 本地完全没有 scene meshes，`scene_prepare` snapshot 也会直接把 runtime `radiance_rgb` 写回 clipmap aggregate sample、per-cell sample 与 dominant sample，并把 runtime `dominant_card_id` 直接写回 dominant-node readback。`gpu_scene_prepare_descriptors_include_runtime_voxel_cells` 与 `hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_cells_move_near_or_far_from_probe` 先证明 runtime-owned cell payload 已经真正进入 unified descriptor buffer 和 shader 消费链，而最新加入的 `hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_radiance_changes_with_fixed_layout`、`hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_changes_with_fixed_layout` 与 `hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_matches_different_card_capture_seed_with_fixed_layout` 则进一步锁死“同一 voxel 布局、只改 runtime `radiance_rgb`、只改 runtime `dominant_card_id`，或者只改该 owner 对应 card-capture seed，GPU completion readback 都必须变化”的合同，说明 GPU seam 已经开始分别消费 runtime voxel color authority、owner authority 与 matched card-capture seed authority，而不是继续只看 synthetic descriptor math。这次新增的 `hybrid_gi_resolve_uses_scene_prepare_voxel_fallback_without_current_trace_schedule` 与 `hybrid_gi_resolve_uses_scene_prepare_voxel_clipmap_fallback_without_runtime_voxel_cells` 则证明 resolve 侧已经会在没有 current trace schedule 时消费 `hybrid_gi_scene_prepare` 的 voxel truth，而且在 runtime 缺失 cell payload 时还能退回 coarse clipmap fallback，而不是继续退回纯黑。最新加入的 `hybrid_gi_resolve_uses_runtime_scene_voxel_tint_when_layout_stays_fixed`、`hybrid_gi_resolve_uses_runtime_scene_voxel_point_light_seed_when_layout_and_tint_stay_fixed` 与 `hybrid_gi_resolve_uses_runtime_scene_voxel_owner_card_capture_seed_when_layout_and_owner_stay_fixed` 又把这条 runtime voxel authority 再往前推了一步：同一套 runtime voxel 布局固定时，不管只改 scene mesh tint、只改 point-light direct seed，还是只改 matched card-capture seed，最终 GI resolve 都会跟着变化，不再把两帧压回同一个空间启发式结果。最新的 `hybrid_gi_resolve_changes_when_runtime_scene_voxel_owner_matches_scene_card_capture_material_seed_with_fixed_layout` 则补上了 resolve 侧最后一段不对称 seam：当 `card_capture_request` 布局、voxel owner 和 per-cell radiance 都保持不变时，只改 scene material truth 也必须让 final resolve 改变；这条回归现在通过，是因为 post-process 已经开始消费当前 frame 的 `scene_prepare_resources.capture_slot_rgba_samples / atlas_slot_rgba_samples`，而不是继续依赖 synthetic request RGB。当前这个 checkpoint 再补上了四条关键回归：`gpu_scene_prepare_descriptors_preserve_explicit_black_runtime_voxel_radiance`、`hybrid_gi_gpu_completion_readback_preserves_explicit_black_runtime_voxel_radiance_with_fixed_layout`、`hybrid_gi_scene_prepare_preserves_explicit_black_runtime_voxel_radiance_without_scene_meshes` 和 `runtime_explicit_black_voxel_radiance_stays_authoritative_over_owner_card_and_spatial_fallback` 证明 runtime-owned `voxel_cells` 已经能用独立 `radiance_present` 位保留显式黑色 authority，而不会再错误退回 owner-card 或 spatial heuristic。这个 checkpoint 现在再补上 clean-frame persisted page truth 的四层收束回归：`hybrid_gi_runtime_state_uses_persisted_surface_cache_page_sample_for_clean_frame_voxel_radiance` 证明 `HybridGiSurfaceCacheState` 中的 resident page capture sample 已经会在下一次 clean-frame scene sync 中回灌匹配 owner page 的 occupied `voxel_cells.radiance_rgb`，不再让 runtime voxel truth 停留在旧的 tint/direct-light placeholder；`hybrid_gi_resolve_uses_runtime_scene_voxel_radiance_rehydrated_from_persisted_page_sample_on_clean_frame` 则进一步证明，即使显式清空 renderer 输入里的 `surface_cache_page_contents`，最终 GI resolve 仍然会随着 warm/cool persisted page sample 改变，而且 `capture_slot_rgba_samples / atlas_slot_rgba_samples` 保持为空，说明这一轮差异已经来自 runtime voxel radiance 本身，而不再只是 owner-card fallback。现在这条 clean-frame seam 又被直接抬进了更广义的 descriptor authority：`collect_inputs_counts_clean_frame_persisted_surface_cache_pages_as_card_descriptors` 与 `gpu_scene_prepare_descriptors_include_clean_frame_persisted_surface_cache_pages` 锁定了“无 dirty request 的 resident persisted page 仍然必须补成 card descriptor”的 staging 合同，而这次新增的 `collect_inputs_skips_absent_clean_frame_persisted_surface_cache_pages_when_counting_card_descriptors`、`gpu_scene_prepare_descriptors_skip_absent_clean_frame_persisted_surface_cache_pages` 与 `gpu_scene_persisted_page_card_capture_seed_rgb_uses_atlas_when_capture_sample_is_absent` 又把 persisted-page presence 语义锁死成和 resolve-side 一致的 contract：`alpha = 0` 代表 truly absent，不得再膨胀 descriptor count 或伪造黑色 seed；如果 `capture_sample_rgba` 缺席但 `atlas_sample_rgba` 仍在，GPU descriptor 必须复用 atlas RGB。`scene_prepare_resources(...)` 现在也按同一规则独立创建 atlas/capture 资源，所以 `hybrid_gi_scene_prepare_absent_persisted_surface_cache_page_contents_do_not_create_resource_snapshot_without_other_scene_prepare_data`、`hybrid_gi_scene_prepare_absent_persisted_surface_cache_page_contents_do_not_occupy_atlas_or_capture_slots`、`hybrid_gi_scene_prepare_atlas_only_persisted_surface_cache_page_contents_do_not_occupy_capture_slots` 和 `hybrid_gi_scene_prepare_capture_only_persisted_surface_cache_page_contents_do_not_occupy_atlas_slots` 共同锁死了“每一 side 只对自身 present sample 负责”的 renderer contract，不再伪造跨 side 占位或零尺寸纹理。runtime 这一侧也已经跟上：`hybrid_gi_runtime_state_keeps_atlas_only_surface_cache_page_samples_across_clean_frames` 与 `hybrid_gi_runtime_state_keeps_capture_only_surface_cache_page_samples_across_clean_frames` 证明 `HybridGiSurfaceCacheState::apply_scene_prepare_resources(...)` 不会再把 one-sided persisted page truth 丢掉，只要 atlas/capture 任一 side 仍然 present，clean-frame runtime reuse 就会把它继续保留下来。`hybrid_gi_gpu_completion_readback_uses_clean_frame_persisted_surface_cache_page_descriptors_without_dirty_requests_or_runtime_voxels` 继续证明即便 runtime voxel radiance 也显式缺席，GPU completion 仍然会随着 clean-frame persisted page sample 改变，说明 `update_completion.wgsl` 已经能直接消费这批 synthetic clean-frame card descriptor，而新加的 `hybrid_gi_gpu_completion_readback_ignores_absent_clean_frame_persisted_surface_cache_pages_without_dirty_requests_or_runtime_voxels` 则从 render-level 证明 truly absent 的 page 会回到 no-page baseline，不再制造 false black GPU authority。这次再往前的一小步则把同一批 page truth 直接抬进了没有 runtime voxel scaffolding 的 resolve：`scene_prepare_persisted_surface_cache_page_samples_provide_spatial_fallback_without_runtime_voxel_support` 在 helper 级证明当 `voxel_clipmaps + voxel_cells` 都为空时，`hybrid_gi_hierarchy_rt_lighting.rs` 已经会按 page bounds 混合 nearby `surface_cache_page_contents`，而 `hybrid_gi_resolve_uses_persisted_surface_cache_page_sample_without_runtime_voxel_support` 则在 render 级证明最终 GI resolve 不会再因为 runtime voxel support 缺席而纯黑，只要 clean-frame persisted page sample 还在场，warm/cool page truth 仍然能穿过最终 composite。这个 checkpoint 现在也补上了剩余的空 trace schedule exact-runtime irradiance seam：`exact_runtime_irradiance_blends_current_surface_cache_truth_when_trace_schedule_is_empty` 证明 `hybrid_gi_hierarchy_irradiance.rs` 不会再让 stale `hierarchy_irradiance` 把 warm/cool clean-frame page truth 压成同一 runtime-only 结果，而 `hybrid_gi_pending_probe_gpu_irradiance_blends_exact_runtime_source_with_current_surface_cache_truth_when_trace_schedule_is_empty` 则把同一约束抬到 pending-probe GPU irradiance readback，防止 helper 合同和高层 encode 行为再次分叉。与此同时，`hybrid_gi_scene_prepare_card_capture_samples_change_with_material_roughness` 与 `hybrid_gi_scene_prepare_voxel_samples_change_with_material_metallic` 又把 card/voxel capture 从 `base_color + emissive` 的最小 seed 推到了 `base_color + emissive + roughness + metallic` 的更丰富 surface-property capture，说明 scene-prepare atlas/voxel 现在已经会对材质表面响应而不只是颜色和自发光做出可观测变化。这个 checkpoint 再往前补上了第一批真实材质纹理：`MaterialCaptureSeed / MaterialRuntime` 现在会保留 `base_color_texture / metallic_roughness_texture / emissive_texture`，`card_capture_shading.rs` 会用稳定的 scene-prepare sample UV 读取 CPU texture asset，并把 `base_color_texture` 乘进 albedo、把 `metallic_roughness_texture` 的 `G/B` 通道乘进 `roughness/metallic`、把 `emissive_texture` 乘进 emissive seed。对应地，`hybrid_gi_scene_prepare_card_capture_samples_change_with_material_base_color_texture`、`hybrid_gi_scene_prepare_card_capture_samples_change_with_material_emissive_texture`、`hybrid_gi_scene_prepare_voxel_samples_change_with_material_emissive_texture` 与 `hybrid_gi_scene_prepare_voxel_samples_change_with_material_metallic_roughness_texture` 现在都通过，说明在标量材质参数固定时，仅修改贴图内容也能稳定改变 atlas/capture/voxel scene-prepare sample，而整组 `graphics::tests::hybrid_gi_scene_prepare_resources` 也重新回到绿色。相邻的 `hybrid_gi_resolve_uses_runtime_gpu_trace_lighting_source_without_current_trace_schedule`、`hybrid_gi_resolve_uses_runtime_hierarchy_rt_lighting_without_current_trace_schedule`、`hybrid_gi_resolve_uses_descendant_scene_driven_runtime_rt_for_parent_probe_after_schedule_clears` 与 `hybrid_gi_resolve_surface_cache` 也继续保持绿色，`cargo check -p zircon_runtime --locked --lib` 通过。

## Current Limits

这仍然不是完整的 Lumen scene pipeline，当前限制需要明确记录：

- surface cache 现在已经有第一层 runtime-owned persistent page content：renderer `scene_prepare` readback 的 atlas/capture sample 会按 resident `page_id` 沉淀进 `HybridGiSurfaceCacheState`，并跨 clean frame / invalidation 维持正确生命周期；这批 sample 现在已经能重新进入 clean-frame `HybridGiScenePrepareFrame`、scene-prepare atlas/capture readback、synthetic clean-frame card descriptor GPU completion、owner-card resolve fallback、无 runtime voxel support 的 page-bounds spatial fallback、hierarchy irradiance fallback，以及 clean-frame runtime voxel radiance rehydration。persisted-page descriptor 路径本身也已经有了显式 presence contract，所以 `alpha = 0` 不会再伪装成黑色 authority，而显式黑色仍保持 authoritative；但它还不是完整的 persistent GPU atlas/page-table residency manager，也还没有把 screen-visible surface-cache hit path正式切到 page reuse
- `card_capture_requests + voxel_clipmaps + voxel_cells` 现在都已经接进 renderer，而且 unified descriptor buffer 也已经开始真实承载这三类 scene-prepare payload；`voxel_cells` 已经不只是 occupancy/count/cell-center truth，还会把 runtime `radiance_rgb` 与 `dominant_card_id` authority 直接打进 descriptor，并分别被 shader 的 color path 与 owner-fallback path 消费；owner-fallback path 本身也会优先复用 matched card-capture seed，但它仍然只是 dominant tint + split direct-light seed 的近场 bias 来源，不是完整的 voxel material/surface cache authority
- voxel scene 现在已经多了一层 runtime-owned fixed-grid `voxel_cells` occupancy/count/dominant-tint contract，再叠加 per-clipmap debug/sample seed、occupancy mask、cell-level volume-content readback、renderer-local dominant contributor ids 与 dominant contributor color truth，并且 resolve 侧已经开始在 trace miss 时把 `voxel_cells` 与 `voxel_clipmaps` 一起用作第一版软件 fallback；但它仍然是 tint-driven + spatial fallback 的 clipmap/cell lighting，不是最终软件 voxelization，也还没有进入真正的 screen-trace hit/miss 合流
- exact runtime irradiance scene truth 目前仍然只由 surface-cache / persisted-page authority 提供；voxel scene 当前负责的是 exact RT miss fallback 与其 temporal reset/change-serial。也就是说，voxel-only fixture 已经会通过 exact RT revision 拒绝旧 GI history，但还不会额外合成一份独立的 exact irradiance scene-truth revision
- `scene_prepare_resources -> resolve` 的 renderer-side voxel sample 路径和 runtime-owned `voxel_cells` 现在都已经有显式 presence contract，显式黑色 sample / radiance authority 不会再被误当成缺失；但它们当前仍然只是 minimal radiance seed，而不是完整的 texture-backed surface cache 内容，所以 resolve miss fallback 还没有进入真正的 page-reuse / surface-property reuse 合流
- renderer-side card/voxel capture 现在已经会同时消费 `base_color + emissive + metallic + roughness` 和首版完整材质纹理集：`base_color_texture / normal_texture / metallic_roughness_texture / occlusion_texture / emissive_texture` 都已经进入 scene-prepare capture；同一条 minimal capture BRDF 现在还会尊重 `double_sided` 与 `alpha_mode(mask/blend)`，所以 backface lighting、cutout reject 与 alpha-blend 衰减不再被错误压成“所有材质都等价于 opaque + double-sided”。这些结果现在已经能沉淀成 runtime-owned persistent page samples，但采样仍然只用稳定中心 UV，也还没有升级成真正的 surface-cache reuse / relight 内容
- 旧 probe / trace-region runtime path 仍然存在于迁移层，主要用于 fixture、runtime host 兼容和旧测试面；不过当 `hybrid_gi_scene_prepare` 已经存在时，scene-driven frame 现在已经不会再让 authored trace-region 直接驱动 final composite 或 RT fallback，也不会再让 authored `resident_probes[].irradiance_rgb`、authored probe `position/radius`、没有 authored source 的 compatibility-only resident probe slots、纯 `probe_id` 变化、纯 descendant probe-id 变化、纯 descendant depth 变化、纯 ancestor depth 变化、或只因插入 authored ancestor 才重新变得 reachable 的 continuation-only hierarchy resolve weight / continuation-only inherited RGB 直接充当最终 GI composite / temporal signature 的颜色、空间、容器、identity 或 intensity 真源。当前残余 authored glue 已经进一步收缩到 descendant/runtime-parent-chain reblend asymmetry 与 compatibility path，而不是 descendant / ancestor depth、continuation-only resolve-weight authority 或 continuation-only inherited RGB shaping 本身
