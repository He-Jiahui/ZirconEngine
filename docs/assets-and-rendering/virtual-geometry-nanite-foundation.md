---
related_code:
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/virtual_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs
  - zircon_runtime/src/asset/assets/model.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/mod.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/cook.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/inspection_dump.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/bvh_graph_dump.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/binary_dump.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_obj.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_resources.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams/diagnostics.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams/metrics.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_execution_draw.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_stream_contract.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_stream_contract/*
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_model.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/runtime/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src/test_support/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/test_support/render_feature_fixtures.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_nanite_cpu.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/declarations/virtual_geometry_page_request.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/declarations/virtual_geometry_runtime_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/plan_ingestion.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/snapshot.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/mod.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/gpu_completion.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_output.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_state.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_stats.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
  - zircon_plugins/virtual_geometry/runtime/src/provider.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/automatic_extract.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/cpu_reference.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/execution_mode.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/runtime_feedback_batch.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot/query_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_mesh_sources/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/virtual_geometry_execution_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/virtual_geometry_submission_detail.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/mod.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/cluster_ids_for_entity.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_count.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_ordinal.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/automatic_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/collect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/ordering.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats/virtual_geometry_indirect_stats.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats/collect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_hardware_rasterization_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/output.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/store_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/execute.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/page_requests.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_visbuffer64_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats_store_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_hardware_rasterization_record_count.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_hardware_rasterization_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_selected_cluster_count.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_selected_cluster_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_selected_clusters.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_visbuffer64.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_visbuffer64_entry_count.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_visbuffer64_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_authority_records.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_execution_indices.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_execution_records.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_execution_segments.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_segments.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_child_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_cluster_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_dispatch_setup_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_global_state_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_hierarchy_child_ids.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_instance_seeds.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_instance_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_launch_worklist_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_traversal_records.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/take_gpu_completion_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/take_gpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_cull.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/debug_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_selected_cluster_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_hardware_rasterization_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_hardware_rasterization_record_count.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_selected_cluster_count.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64_entry_count.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_selected_clusters.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_cull_input_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_instance_seeds.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_instance_work_items.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_cluster_work_items.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_child_work_items.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_hierarchy_child_ids.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_traversal_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_global_state_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_dispatch_setup_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_launch_worklist_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_input_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_selected_cluster_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_words.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_selected_clusters.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/indirect_counts/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_render_path_summary.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/take_gpu_completion_parts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/bind_group_layout.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/uploader_pipeline.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/params_buffer.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/node_and_cluster_cull_instance_work_item_pipeline.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/virtual_geometry_gpu_resources.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/node_and_cluster_cull_instance_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/shaders/node_and_cluster_cull_instance_work_items.wgsl
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/decode/read_buffer_u32s.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/pending_readback/collect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/accessors.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/completion.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback_completion_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/render_path_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/runtime_rt_sources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/scene_prepare_rt_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/scene_prepare_voxel_samples.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/trace_region_inheritance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_temporal_signature.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/pending_readback/collect.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_node_and_cluster_cull_execution.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_raster_draw.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_selection.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_node_and_cluster_cull_child_work_item.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_node_and_cluster_cull_cluster_work_item.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_node_and_cluster_cull_traversal_record.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_selection.rs
  - zircon_runtime/src/scene/world/render.rs
implementation_files:
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/asset/assets/model.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/mod.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/cook.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/inspection_dump.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/bvh_graph_dump.rs
  - zircon_runtime/src/asset/virtual_geometry_cook/binary_dump.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_obj.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_resources.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams/diagnostics.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams/metrics.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_execution_draw.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_model.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/runtime/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src/test_support/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/test_support/render_feature_fixtures.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/declarations/virtual_geometry_page_request.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/declarations/virtual_geometry_runtime_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/plan_ingestion.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/snapshot.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/mod.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/gpu_completion.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_output.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_state.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_stats.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
  - zircon_plugins/virtual_geometry/runtime/src/provider.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/automatic_extract.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/cpu_reference.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/execution_mode.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/runtime_feedback_batch.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot/query_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_mesh_sources/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/virtual_geometry_execution_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/virtual_geometry_submission_detail.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/mod.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/cluster_ids_for_entity.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_count.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_ordinal.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/automatic_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/collect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/ordering.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats/virtual_geometry_indirect_stats.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats/collect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_hardware_rasterization_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/output.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/store_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/execute.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/page_requests.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_visbuffer64_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/debug_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_cull_input_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_selected_cluster_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_hardware_rasterization_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_hardware_rasterization_record_count.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_selected_cluster_count.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64_entry_count.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_selected_clusters.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_instance_seeds.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_instance_work_items.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_cluster_work_items.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_child_work_items.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_hierarchy_child_ids.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_traversal_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_global_state_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_dispatch_setup_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_launch_worklist_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_input_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_node_and_cluster_cull_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_selected_cluster_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_words.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_selected_clusters.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_render_path_summary.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/take_gpu_completion_parts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/bind_group_layout.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/uploader_pipeline.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/params_buffer.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/new/node_and_cluster_cull_instance_work_item_pipeline.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/virtual_geometry_gpu_resources.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/virtual_geometry_gpu_resources/node_and_cluster_cull_instance_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/shaders/node_and_cluster_cull_instance_work_items.wgsl
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/decode/read_buffer_u32s.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/pending_readback/collect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/accessors.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/completion.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback_completion_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/readback/virtual_geometry_gpu_readback/render_path_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_temporal_signature.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/runtime_rt_sources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/scene_prepare_rt_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/scene_prepare_voxel_samples.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/trace_region_inheritance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/pending_readback/collect.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_node_and_cluster_cull_execution.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_raster_draw.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_selection.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_node_and_cluster_cull_child_work_item.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_node_and_cluster_cull_cluster_work_item.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_node_and_cluster_cull_traversal_record.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_selection.rs
  - zircon_runtime/src/scene/world/render.rs
plan_sources:
  - user: 2026-04-21 implement the M5 Nanite-like Virtual Geometry convergence plan
  - docs/superpowers/specs/2026-05-01-plugin-renderer-hard-cutover-design.md
  - docs/superpowers/plans/2026-05-01-plugin-renderer-hard-cutover.md
  - docs/superpowers/specs/2026-05-02-plugin-renderer-neutral-readback-execution-surface-design.md
  - docs/superpowers/plans/2026-05-02-plugin-renderer-neutral-readback-execution-surface.md
  - .codex/plans/GI_VG 插件化激进迁移计划.md
  - .codex/plans/zircon_plugins 全量插件化收敛规划.md
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
  - docs/superpowers/plans/2026-05-01-shared-renderer-fixture-localization.md
tests:
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/virtual_geometry_cook.rs
  - zircon_runtime/src/asset/tests/assets/model.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_imported_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/tests/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/tests/support.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/tests/selection_filter.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/tests/seed_backed_ranges.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/tests/seed_backed_fallbacks.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_executed_cluster_selection_pass/tests/seed_backed_ordering.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/output.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/store_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_node_and_cluster_cull_pass/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_visbuffer64_pass/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_render_passes/virtual_geometry_indirect_stats_store_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/mod.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_hardware_rasterization_record_count.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_hardware_rasterization_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_selected_cluster_count.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_selected_cluster_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_selected_clusters.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_visbuffer64.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_visbuffer64_entry_count.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_gpu_readback_visbuffer64_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_authority_records.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_execution_indices.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_execution_records.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_execution_segments.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_indirect_segments.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_child_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_cluster_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_dispatch_setup_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_global_state_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_hierarchy_child_ids.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_instance_seeds.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_instance_work_items.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_launch_worklist_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_source.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_traversal_records.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/take_gpu_completion_parts.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/take_gpu_readback.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/test_support/render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_node_and_cluster_cull_execution.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_stats.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_nanite_cpu.rs
  - zircon_runtime/tests/virtual_geometry_extract_contract.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_stream_contract.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_stream_contract/*
  - zircon_runtime/tests/virtual_geometry_execution_snapshot_contract.rs
  - zircon_runtime/tests/virtual_geometry_visbuffer_overlay_contract.rs
  - zircon_runtime/tests/virtual_geometry_visibility_debug_contract.rs
  - zircon_runtime/tests/virtual_geometry_stats_contract.rs
  - zircon_runtime/tests/support/mod.rs
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline render_feature_fixture -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --offline
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline virtual_geometry_nanite -- --nocapture
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture
  - stale-path search gate for crate::graphics::types, crate::graphics::scene::scene_renderer, pub(in crate::graphics...), crate::graphics::runtime, and crate::graphics::tests under zircon_plugins/virtual_geometry/runtime/src/virtual_geometry
  - broad stale-path search gate for crate::graphics:: under zircon_plugins/virtual_geometry/runtime/src/virtual_geometry
  - cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml --offline
  - cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never
  - git diff --check
doc_type: module-detail
---

# Virtual Geometry Nanite Foundation

## 2026-04-25 RenderFeature Integration

Virtual Geometry 最初通过 `BuiltinRenderFeature::VirtualGeometry` 声明 prepare、node/cluster cull、page feedback、visbuffer/hardware fallback、debug overlay 等 RenderGraph pass descriptor；Stage 4 插件化后，这些高级 pass 不再由基础 renderer 隐式打开，而是由 linked `virtual_geometry` render descriptor 加上 `VirtualGeometry` capability gate 进入 compiled graph。重型 runtime state、Nanite CPU reference 与 page residency host 已物理迁到 `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/`；`zircon_runtime` 只保留中立 render DTO、asset/cook、base renderer、debug snapshot surface、descriptor/capability gate，以及 `graphics::virtual_geometry_runtime_provider` 里的 provider/state/feedback/prepare/stat contracts。当前 base submit path 会通过 linked plugin registration 创建 erased `VirtualGeometryRuntimeState`，再把 extract、visibility plan、visible clusters、draw segments、renderer GPU completion、visibility feedback 和 page requests 交给 `PluginVirtualGeometryRuntimeProvider`，所以 concrete residency/slot/pending/hot-frontier state 仍归插件 crate，而 render framework 只保存中立 trait object。

因此 public integration contracts 也不能再用裸 `WgpuRenderFramework::new(...)` 加 `with_virtual_geometry(true)` 作为高级 VG 前提。`zircon_runtime/tests/support/mod.rs` 现在提供 `virtual_geometry_wgpu_render_framework(...)`，让 debug snapshot、stats、execution snapshot 和 visbuffer overlay integration tests 走与 production plugin cutover 一致的 descriptor-linked framework setup，而不是重新引入旧 built-in feature identity。

## 2026-05-01 Virtual Geometry Owner Cutover

Virtual Geometry prepare、cluster selection、node/cluster cull、cluster raster draw、scratch/debug completion DTO 不再从 `zircon_runtime/src/graphics/types` 导出，新的直接导入路径是 `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/`。原 runtime `graphics::runtime::virtual_geometry` owner、root scene renderer `virtual_geometry` GPU resources/readbacks/shaders、Nanite runtime helpers、VG render pass source、root-output conversion helpers 与旧 root graphics VG tests 已迁入 `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/`。

`zircon_runtime` 继续保留 asset/cook/public extract、core framework debug snapshot DTO、base renderer mesh draw projection，以及 `graphics::virtual_geometry_runtime_provider` 的 erased provider/state/feedback/prepare/stat contract；concrete residency、slot allocator、pending page state、page-table completion、prepare output 和 GPU pass readback parts 都由插件私有持有。Root render path 只通过 linked descriptor/executor/capability gate 和中立 render/debug DTO 观察 VG。

验证记录：迁移阶段使用 `cargo check -p zircon_runtime --lib --locked --offline` 与 `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --offline`，两者均已通过；最终单元测试已在 GI/VG 代码与文档迁移完成后执行，`cargo test -p zircon_runtime --lib --locked --offline` 通过 562/562，插件 runtime 测试 `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline` 通过 Virtual Geometry 8/8。2026-05-01 的 plugin test-source localization 继续把 Nanite CPU/reference regression 挂回 `zircon_plugin_virtual_geometry_runtime`，并通过 `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture` 验证 30/30。

这次 localization 没有恢复 `zircon_runtime::graphics::runtime::virtual_geometry` 或旧 root `zircon_runtime/src/graphics/tests/virtual_geometry_nanite_cpu.rs` owner。测试直接使用插件 runtime 的 `crate::virtual_geometry::*` Nanite helpers，并只从 `zircon_runtime::core::framework::render`、asset、math、resource、scene DTO surface 读取中立 contracts。renderer/root pass integration sources 仍按 hard-cutover 原则留在插件 `renderer/` tree 中，但只在 stale private `scene_renderer` visibility 和 old `graphics::types` imports 被切掉后再接入。

2026-05-01 的 shared fixture localization 继续把 moved renderer test-source fixture dependency 收到插件 crate 内部：`zircon_plugins/virtual_geometry/runtime/src/test_support/render_feature_fixtures.rs` 直接调用 `crate::render_feature_descriptor()` 与 `crate::virtual_geometry_runtime_provider_registration()`，test sources 只导入 `crate::test_support::render_feature_fixtures::*`，不再引用 `zircon_runtime::graphics::tests::plugin_render_feature_fixtures`。VG plugin runtime source tree 对旧 fixture 路径的搜索结果为 zero hits；本轮 scoped evidence 为 `render_feature_fixture` 1/1、`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline` 通过，以及完整插件包测试 30/30 通过。

## 2026-05-01 Plugin Renderer Hard-Cutover Follow-Up

The plugin runtime crate owns concrete renderer resources, readbacks, pass helpers, and feature-specific prepare/resolve DTOs. `zircon_runtime` only exposes neutral graphics/frame/render-graph contracts used by plugin registration and execution boundaries. Old `zircon_runtime::graphics::runtime::*` and `zircon_runtime::graphics::scene::scene_renderer::{hybrid_gi,virtual_geometry}` owner paths are not compatibility surfaces.

VG renderer ownership now compiles through plugin-local renderer wiring without publicizing runtime-private `MeshDraw`. The neutral `RenderVirtualGeometryExecutionDraw` view is defined under `zircon_runtime::core::framework::render` and is projected from `MeshDraw` inside runtime-owned mesh-draw code; plugin renderer passes consume that neutral draw view instead of naming scene-renderer internals. GPU readback decode also owns a plugin-local `read_buffer_u32s` helper, so moved readback code does not reopen a runtime-private backend helper as a public API.

Broad moved renderer tests that still depend on runtime-private `SceneRenderer`, `RenderBackend`, `ResourceStreamer`, or old frame extension methods remain intentionally unwired. Promoting those tests requires a deliberate neutral readback/execution surface first; this cutover records package-level plugin tests and stale-path search gates as the current acceptance boundary rather than restoring old owner paths.

Milestone 5 closeout evidence was refreshed on 2026-05-02 with `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture`, which ran 52 library tests with 0 failures. Scoped old-owner searches for the exact `crate::graphics::types`, `crate::graphics::scene::scene_renderer`, `pub(in crate::graphics...)`, `crate::graphics::runtime`, and `crate::graphics::tests` patterns returned no files under `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry`; the broader `crate::graphics::` search also returned no files. This evidence accepts package-level renderer ownership and intentionally does not promote the deferred moved renderer tests that still need a neutral readback/execution API.

The 2026-05-02 neutral renderer follow-up now wires `root_state_readbacks` from `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs` after converting the moved readback helpers into plugin-local functions over `VirtualGeometryGpuReadback` and `VirtualGeometryGpuReadbackCompletionParts`. `root_render_passes/mod.rs` remains a structural renderer root and only exposes renderer-scoped store-parts seams needed by sibling plugin modules, including `VirtualGeometryIndirectStats`, `VirtualGeometryIndirectStatsStoreParts`, and `VirtualGeometryNodeAndClusterCullPassStoreParts`; this is not a compatibility re-export for the deleted runtime owner path. Focused evidence refreshed `zircon_plugins/Cargo.lock`, formatted the VG plugin, passed `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never`, and passed `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never` with 52 tests and 0 failures. Workspace-wide validation is not implied by this package-level evidence.

## What This Slice Adds

This change lands the first Zircon-native Nanite-like foundation without replacing the current M5 Virtual Geometry runtime.

The implemented scope is intentionally the lowest stable layer:

- `ModelPrimitiveAsset` can now carry an optional cooked `virtual_geometry` payload beside the legacy vertex/index mesh.
- The cooked payload has a stable, typed schema for hierarchy nodes, cluster headers, cluster page headers, raw page bytes, root page information, and debug metadata.
- A CPU reference path can traverse the hierarchy, enumerate leaves, filter by `forced_mip`, track resident pages, and bridge the selected clusters into the existing `RenderVirtualGeometryExtract` shape.
- A first execution-mode taxonomy is defined for later runtime routing:
  - `CpuDebug`
  - `BaselineGpu`
  - `FlagshipGpu`

This section records the first N1/N2 foundation from the Nanite plan. Later sections below document the baseline/stub N3 seams that now feed `NodeAndClusterCull`, logical `VisBuffer64`, selected-cluster buffers, and hardware-raster startup records; the remaining N3-N7 gaps are the real pixel-addressable `VisBuffer64`, `ClusterPageData` decode, shader hardware rasterization, full GPU BVH traversal, and UE5.5 comparison path.

## Zircon-Native Cook Seed

The newest N1 continuation adds the first deterministic mesh-to-`VirtualGeometryAsset` cook entry point at `zircon_runtime/src/asset/virtual_geometry_cook/cook.rs`.

`cook_virtual_geometry_from_mesh(...)` keeps the base mesh intact and returns an optional cooked VG payload that can be attached to `ModelPrimitiveAsset.virtual_geometry`. Its first implementation is intentionally inspection-friendly rather than compression-optimal:

- validates indexed triangle input before emitting cooked data,
- slices source triangles into stable leaf clusters using `VirtualGeometryCookConfig.cluster_triangle_count`,
- recursively groups children with a four-way BVH fanout,
- emits `VirtualGeometryHierarchyNodeAsset` and `VirtualGeometryClusterHeaderAsset` rows with parent linkage, bounds, mip level, and monotonic screen-space error,
- writes one explanatory little-endian page payload per node and matching `VirtualGeometryClusterPageHeaderAsset` metadata,
- publishes `root_page_table`, `root_cluster_ranges`, mesh/source labels, and a cook note in `VirtualGeometryDebugMetadataAsset`.

This is the first concrete Zircon-authored cook chain for the N1 plan. It does not yet replace importer policy, run mesh simplification, optimize page compression, or consume UE Nanite blobs; those remain later cook-tooling and UE5.5 comparison stages. The important boundary is that offline data now originates in the asset subsystem and feeds the same `VirtualGeometryAsset` schema already used by CPU reference, automatic extract, runtime residency, and renderer debug seams.

Validation for this slice is `zircon_runtime/src/asset/tests/virtual_geometry_cook.rs`, which checks deterministic repeat cook output, four-way fanout, root page/range wiring, page payload/header size consistency, monotonic parent error, model TOML roundtrip with the base mesh preserved, and invalid input rejection.

The importer bridge now makes that cook seed the default asset-ingest authority for imported model primitives. `primitive_from_indexed_mesh(...)` attaches cooked `VirtualGeometryAsset` data for OBJ and GLTF primitives while preserving the base vertices and indices as fallback/render comparison data. `.model.toml` imports call the same cook path only for primitives that do not already contain `virtual_geometry`, so authored or previously cooked payloads remain authoritative while uncooked model TOML files are backfilled deterministically from their stored mesh data. The cook debug metadata records the resource URI as `source_hint`, allowing the automatic extract and CPU-reference debug streams to trace live VG instances back to the imported model asset.

The paired extraction proof now lives under the Virtual Geometry runtime plugin owner rather than restoring the deleted `zircon_runtime::graphics::runtime::virtual_geometry` path. `build_virtual_geometry_automatic_extract_from_meshes_with_debug(...)` already loads `ModelAsset` values and collects primitives with `virtual_geometry`. The new plugin-owned regression path imports an OBJ through `AssetImporter`, feeds the resulting cooked `ModelAsset` through the mesh-based automatic extract helper, and asserts that the synthesized `RenderVirtualGeometryInstance` carries the scene entity, source model id, source hint, non-empty clusters/pages, and CPU-reference debug instance. This proves scene extraction can source live VG instances from cooked imported assets without adding SRP/RHI/pass-resource changes or compatibility re-exports.

The newest N1 continuation adds `format_virtual_geometry_cook_inspection_dump(...)` in `zircon_runtime/src/asset/virtual_geometry_cook/inspection_dump.rs`. The dump is intentionally text and deterministic so teaching tools, acceptance logs, and later CPU-reference comparisons can inspect cooked data without linking renderer-private debug helpers. It writes:

- cook dump version, debug mesh/source labels, notes, and top-level counts,
- root page ids and root cluster ranges,
- hierarchy nodes sorted by node id, including parent, mip, page, cluster range, children, bounds, and screen-space error,
- cluster headers sorted by cluster id,
- leaf-cluster rows derived from hierarchy child counts,
- cluster ids grouped by mip level,
- cluster ids grouped by page id,
- page headers plus decoded explanatory payload header/items from the first Zircon-native cook payload format.

This still is not a compressed production page format or a UE Nanite blob parser. It is the N1 inspection/export surface promised by the milestone plan: a stable view of BVH shape, leaf clusters, Mip distribution, page mapping, and page payload summaries that can be checked into logs or compared against the later single-thread CPU reference without stepping into SRP/RHI, runtime residency, or GPU pass ownership.

The N1 export surface now also includes two focused asset-layer formats:

- `format_virtual_geometry_cook_bvh_graph_dump(...)` writes a deterministic DOT-style BVH graph with one node per hierarchy row, parent-to-child edges, mip/page labels, owning cluster ids, and leaf/internal shape differences for editor tooling or offline teaching diagrams.
- `encode_virtual_geometry_cook_binary_dump(...)` writes a deterministic little-endian inspection binary beginning with `ZVGB`, version/count headers, debug labels/notes, sorted hierarchy rows, sorted cluster headers, sorted page headers, root tables/ranges, and raw page payload bytes.

These exports deliberately mirror the existing cooked `VirtualGeometryAsset` schema rather than introducing a second production page format. They are intended for stable inspection, regression logs, and later CPU/UE comparison tooling; they are not yet compressed cluster streaming blobs or renderer-owned GPU upload packets.

The cook regression suite now also checks that the text dump is repeatable and contains the expected hierarchy, leaf, mip, page-map, and decoded payload rows for the five-triangle fixture. Additional assertions cover deterministic DOT graph structure and deterministic binary header/metadata/node encoding.

The current continuation also crosses the first concrete render-debug boundary from N0/N3: `visualize_bvh` and `visualize_visbuffer` are no longer inspection-only. They now reach the shared SRP overlay path and change the captured frame without introducing a second VG/Nanite query API.

## Unified ClusterSelection Worklist

The latest convergence step removes the last meaningful split between the prepare-owned debug worklist and the runtime-frame-owned fallback raster worklist.

`zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_selection.rs` now defines one internal `VirtualGeometryClusterSelection` DTO that carries both classes of data at once:

- cluster identity and debug-facing fields
  - `instance_index`
  - `entity`
  - `cluster_id`
  - `cluster_ordinal`
  - cluster-local `page_id` / `lod_level`
- submission and raster-facing fields
  - `submission_index`
  - submission `page_id` / `lod_level`
  - `entity_cluster_start_ordinal`
  - `entity_cluster_span_count`
  - `entity_cluster_total_count`
  - `lineage_depth`
  - `frontier_rank`
  - `resident_slot`
  - `submission_slot`
  - `state`

`VirtualGeometryPrepareFrame::cluster_selections(...)` is now the prepare-layer authority. The old baseline outputs are projections from that single source:

- `selected_clusters(...)`
  - projects to the public `RenderVirtualGeometrySelectedCluster` debug DTO
- `same_frame_visbuffer_debug_marks(...)`
  - projects current-frame visbuffer marks from the same selection list
- `cluster_raster_draws(...)`
  - projects fixed-fanout fallback raster submissions by deduplicating submission records instead of rebuilding a second draw list from scratch

`ViewportRenderFrame` now carries `virtual_geometry_cluster_selections` instead of a pre-expanded `virtual_geometry_cluster_raster_draws` map. `build_runtime_frame.rs` snapshots the prepare-owned selection list onto the runtime frame, and `build_virtual_geometry_cluster_raster_draws.rs` derives teaching-path raster draws from that same frame-owned selection seam when prepare is absent.

This matters for the M5 plan because it establishes the first true internal `ClusterSelection` bridge for N3:

- the public debug surface still sees stable `selected_clusters` and `visbuffer_debug_marks`
- the teaching raster path still receives `VirtualGeometryClusterRasterDraw`
- future `HardwareRasterization` / `VisBuffer64` work can now consume one authoritative runtime-frame worklist instead of choosing between a debug DTO and a raster DTO

### Execution Ownership Continuity

This continuation pushes the same `instance_index` ownership one step deeper into the submission and execution chain instead of letting later stages recover it from `entity + cluster range` heuristics.

- `VirtualGeometryClusterRasterDraw` now keeps `instance_index` when `VirtualGeometryClusterSelection` projects into the baseline raster DTO.
- `VirtualGeometryIndirectSegmentKey` and `VirtualGeometrySubmissionDetail` now preserve that same `instance_index`, so the shared indirect-args layout remains the authority for per-instance submission ownership rather than treating instance identity as debug-only metadata.
- `RenderVirtualGeometryExecutionSegment` now exposes `instance_index` on the public renderer-owned snapshot, so execution-facing inspection can follow the same instance lineage as `selected_clusters` and `visbuffer_debug_marks`.
- `store_last_runtime_outputs.rs` now prefers `execution_segments.instance_index` when rebuilding post-render `selected_clusters`, only falling back to extract-time lookup if the execution seam did not carry explicit instance ownership.
- `virtual_geometry_indirect_args.wgsl` now writes `instance_index` into `SubmissionAuthorityRecord`, widening the GPU authority record from 14 to 15 words so later readback can treat the shader-authored authority buffer as a real per-instance fact source instead of a draw-ref-only side channel.
- `read_indirect_segments.rs` now matches the real shared indirect segment layout at 13 words and exposes `read_last_virtual_geometry_indirect_segments_with_instances()`, so shared segment readback preserves both `submission_index` and `instance_index` instead of silently dropping them during host inspection.
- `read_indirect_authority_records.rs` now decodes that widened authority buffer into a typed `VirtualGeometryIndirectAuthorityRecord`, and `read_indirect_execution_segments.rs` can now rebuild typed execution segments either from the dedicated execution-authority buffer or from `execution indices + authority records` when the older shared segment and draw-ref buffers are gone.
- `virtual_geometry_submission_execution_order.rs` now authors explicit `RenderVirtualGeometryInstance` metadata in its fixtures, so the GPU authority/readback path is validated against true `instance_index` ownership instead of relying on entity-only coincidence.
- `RenderVirtualGeometrySubmissionRecord` now also carries `instance_index`, and `store_last_runtime_outputs.rs` backfills that field from execution-backed `original_index -> instance_index` ownership before the renderer persists the public snapshot. Host-visible submission inspection now follows the same per-instance authority chain as `execution_segments`, instead of forcing tooling to merge two debug surfaces manually.
- `RenderVirtualGeometrySubmissionEntry` now mirrors that same ownership on the public submission-order surface. `virtual_geometry_indirect_stats.rs` collects submission order as `(instance_index, entity, page_id)`, and `store_last_runtime_outputs.rs` persists it directly into the renderer-owned snapshot so hosts can inspect actual submission order per instance without joining against `submission_records`.
- The renderer test/readback helpers now expose the same continuity instead of stopping at entity/page tuples. `read_mesh_draw_submission_records.rs` adds `read_last_virtual_geometry_mesh_draw_submission_records_with_instances()`, which preserves `(instance_index, entity, page_id, submission_index, draw_ref_rank)` from stored order, execution segments, GPU authority fallback, or the final shared `submission + draw-ref + segment` fallback. `read_mesh_draw_submission_order.rs` adds the parallel `read_last_virtual_geometry_mesh_draw_submission_order_with_instances()` projection for submission-order inspection.
- `VirtualGeometrySubmissionDetail` is now a MeshDraw-internal submission-truth DTO rather than a renderer-core input type. `mesh_draw/virtual_geometry_execution_projection.rs` owns the conversion from that detail into execution segments, submission order/records, token records, executed-selection keys, and draw-ref indices, so `virtual_geometry_indirect_stats/` and executed-cluster selection consume MeshDraw projections instead of naming the DTO directly.
- `RenderVirtualGeometryVisBuffer64Entry` now gives the renderer-owned snapshot its first true `VisBuffer64` abstraction. `build_virtual_geometry_debug_snapshot.rs` and `store_last_runtime_outputs.rs` both pack execution-facing `selected_clusters` into stable 64-bit visibility entries plus a published `clear_value = 0`, so the host can inspect a real 64-bit visibility result contract before the hardware raster path exists.
- `VirtualGeometryGpuReadback` now carries that same logical `VisBuffer64` result stream whenever a VG GPU prepare/readback pass ran for the frame. Its folder-backed owner keeps the state declaration in `virtual_geometry_gpu_readback/mod.rs`, exposes read access from `accessors.rs`, keeps completion handoff in `completion.rs`, and stores render-path fill/replace behavior in `render_path_writeback.rs`; `pending_readback/collect.rs` seeds those fields on uploader readback creation, and `store_last_runtime_outputs.rs` backfills the execution-backed entry list from the same post-render cluster subset used by the renderer-owned snapshot. `read_gpu_readback_visbuffer64.rs` adds a non-consuming last-state helper for tests and future inspection tooling, so the runtime side can read the same 64-bit visibility contract without forcing tools to query the snapshot path first or consume the stored GPU readback object. The `None` semantics for frames that never produced a VG GPU readback object remain unchanged.
- That logical stream now also lands in a real renderer-owned GPU buffer instead of existing only as DTOs. `store_last_runtime_outputs.rs` packs the final `visbuffer64_entries` into `u64` words, creates `last_virtual_geometry_visbuffer64_buffer`, and stores matching `clear_value` plus `entry_count` on `SceneRenderer`. `read_visbuffer64_words.rs` reads those packed words back even after the test-only `take_last_virtual_geometry_gpu_readback()` inspection path has consumed the CPU DTO; production completion now consumes only `take_last_virtual_geometry_gpu_completion_parts()`, whose `VirtualGeometryGpuReadbackCompletionParts` handoff is visible only inside `crate::graphics`. This fixes the first true buffer boundary needed before a later `HardwareRasterizationPass` can become the producer.
- The newest follow-up makes that buffer's provenance explicit and moves the baseline producer into a named render-path seam. `RenderVirtualGeometryVisBuffer64Source` now distinguishes `Unavailable`, `RenderPathClearOnly`, `RenderPathExecutionSelections`, `SnapshotFallback`, and `GpuReadbackFallback`; `RenderVirtualGeometryDebugSnapshot`, `SceneRenderer`, and `read_visbuffer64_source.rs` preserve that source so tests can prove the packed buffer came from render-path execution ownership instead of an opaque late backfill. The executed-submission filtering, cluster deduplication, stable ordering, and `u64` buffer creation now live in `virtual_geometry_visbuffer64_pass.rs` as `VirtualGeometryVisBuffer64PassOutput`, which `render.rs` consumes directly while `virtual_geometry_indirect_stats.rs` keeps only the accounting role. `render_frame_with_pipeline.rs` now threads that explicit render-path source into `store_last_runtime_outputs.rs`, so a frame that ran the baseline `VisBufferClear` path but produced zero cluster selections remains observable as `RenderPathClearOnly` instead of collapsing to `Unavailable`. This is still a baseline producer, not hardware rasterization, but it is now an explicit pass boundary that can later be replaced by `VisBufferClear + HardwareRasterization` without changing the renderer-owned last-state contract.
- The latest follow-up does the same thing one stage earlier for the future raster handoff. `virtual_geometry_hardware_rasterization_pass.rs` now emits execution-backed `RenderVirtualGeometryHardwareRasterizationRecord` rows directly from the same `ClusterSelection + executed submission key` seam as `VisBuffer64`, preserving cluster identity plus the startup parameters the later raster path will need: submission page/lod, cluster span/total count, lineage depth, frontier rank, and slot ownership. `virtual_geometry_indirect_stats.rs` carries those records as an explicit pass output, and `store_last_runtime_outputs.rs` now persists them onto `RenderVirtualGeometryDebugSnapshot.hardware_rasterization_records`. This keeps the public contract fixed for `ClusterSelection -> HardwareRasterizationPass` even though the producer is still baseline-side and shader rasterization has not landed yet.
- That raster-startup contract now also has explicit provenance and the same real buffer boundary as `VisBuffer64`. `RenderVirtualGeometryHardwareRasterizationSource` now distinguishes `Unavailable`, `RenderPathClearOnly`, and `RenderPathExecutionSelections`; `virtual_geometry_hardware_rasterization_pass.rs` owns the current baseline producer and returns `source + record_count + buffer` directly; and `render.rs`, `render_frame_with_pipeline.rs`, and `store_last_runtime_outputs.rs` thread that output straight into `SceneRenderer` plus `RenderVirtualGeometryDebugSnapshot.hardware_rasterization_source`. `read_hardware_rasterization_source.rs` then exposes the renderer-owned last-state helper for tests, so clear-only frames remain observable even when snapshot assembly is absent. The pass still packs startup records into GPU-readable `u32` words, but the important change is that the buffer is now pass-owned and its provenance is no longer reconstructed later from DTO presence alone.
- The same renderer-owned seam is now visible on the framework stats surface instead of stopping at snapshot/readback inspection. `RenderStats` now carries `last_virtual_geometry_visbuffer64_source`, `last_virtual_geometry_visbuffer64_entry_count`, `last_virtual_geometry_hardware_rasterization_source`, and `last_virtual_geometry_hardware_rasterization_record_count`; `read_render_path_summary.rs` exposes the corresponding `SceneRenderer` getters; and `update_stats/virtual_geometry_stats.rs` forwards those values whenever the current frame has an effective VG extract. The important semantic choice is that stats still reset to `Unavailable`/`0` when the effective VG payload disappears, even if the underlying renderer ran a baseline clear-only pass because the feature stayed enabled. That keeps `RenderStats` aligned with “effective VG workload this frame” instead of leaking renderer-local pass housekeeping into the public stats contract.
- The newest convergence slice removes the last duplicated execution filtering below those baseline producers. `virtual_geometry_executed_cluster_selection_pass.rs` now computes one `VirtualGeometryExecutedClusterSelectionPassOutput` from `ClusterSelection + indirect execution draws`, locking executed submission-key filtering, `(entity, cluster_id)` deduplication, and stable ordering in one unit-tested seam. `virtual_geometry_indirect_stats.rs` executes that seam exactly once per frame, and both `virtual_geometry_visbuffer64_pass.rs` and `virtual_geometry_hardware_rasterization_pass.rs` now consume the shared ordered cluster list instead of each rebuilding the same filter/sort layer independently. That keeps current baseline behavior unchanged while turning the future `NodeAndClusterCull -> HardwareRasterization -> VisBuffer64` path into a producer swap instead of another three-way logic re-alignment.
- The latest follow-up gives that same shared seam its first real renderer-owned GPU buffer boundary instead of leaving it as a transient CPU-only `Vec<VirtualGeometryClusterSelection>`. `RenderVirtualGeometrySelectedCluster` now packs to and from a compact GPU word layout; `virtual_geometry_executed_cluster_selection_pass.rs` now returns `selected_cluster_count + selected_cluster_buffer` beside the internal ordered selections; `render.rs`, `render_frame_with_pipeline.rs`, and `store_last_runtime_outputs.rs` preserve that buffer on `SceneRenderer`; and `read_selected_clusters.rs` decodes it back into typed selected-cluster records for tests. This matters because the exact execution-owned cluster identity stream can now survive even when there is no renderer-owned snapshot and no uploader readback DTO, which is the same “real buffer before real shader producer” pattern already used for `VisBuffer64` and hardware-raster startup records.
- The newest follow-up makes the seed-backed fixed-fanout fallback seam semantically split instead of forcing one record to impersonate both the requested child work and the resident cluster that can actually draw. `virtual_geometry_executed_cluster_selection_pass.rs` now keeps paired internal `selection + selected_cluster` records: `selection` preserves the original child submission metadata (`submission_page_id`, `submission_lod_level`, `entity_cluster_start_ordinal`, `lineage_depth`, `frontier_rank`, and unresolved child state), while the pass-owned `selected_clusters` buffer publishes the resolved resident cluster identity that later feeds `VisBuffer64` and hardware rasterization. When multiple seed-expanded child requests resolve to the same resident ancestor, the seam now preserves stable selected-cluster order but overwrites the older submission metadata with the later child request, so baseline raster startup records report the actual draw target together with the most specific outstanding child request that forced the fallback. The adjacent follow-up then closes the budget-ordering hole that this split exposed: seed-backed root-seed baseline selection now sorts by resolved selected-cluster identity first, clamps `cluster_budget` only after that stable order exists, and refreshes page-based `frontier_rank` from the final emitted worklist rather than from the raw unsorted extract walk. That means unsorted instance slices, duplicate resident-parent fallback, `VisBuffer64`, and hardware-raster startup records now all observe one consistent execution order instead of each implicitly inheriting whichever raw traversal order happened to run first. This is the first place where the current `NodeAndClusterCull -> executed-cluster` bridge behaves like a Nanite-style “requested cluster vs. drawn fallback cluster” seam instead of a teaching-path approximation.
- The newest follow-up finally makes `NodeAndClusterCull` consume that widened startup/global-state seam into the first real per-instance worklist contract instead of stopping at one `NaniteGlobalStateBuffer`-style record. `RenderVirtualGeometryNodeAndClusterCullInstanceSeed` now defines a stable GPU word layout for `(instance_index, entity, cluster_offset/count, page_offset/count)`; `virtual_geometry_node_and_cluster_cull_pass.rs` builds one seed row per effective `RenderVirtualGeometryExtract.instance` while clamping to the typed `cull_input.instance_count`; and `render.rs`, `render_frame_with_pipeline.rs`, and `store_last_runtime_outputs.rs` preserve `instance_seed_count + instance_seed_buffer` on `SceneRenderer` beside the existing global-state buffer. `read_node_and_cluster_cull_instance_seeds.rs` then exposes that renderer-owned seed buffer for tests and future tooling, so the current bridge no longer says only “here is the camera and budgets” but also “here is the concrete root worklist the later GPU `VisitNode / StoreCluster` traversal should start from.” This is still not BVH traversal, but it is the first explicit `global_state -> worklist seed` seam in the render path.
- The same follow-up now reaches the public inspection surfaces instead of stopping at renderer-private readback. `RenderVirtualGeometryDebugSnapshot` carries `node_and_cluster_cull_instance_seeds`, seeded as empty at submission-build time and backfilled from the render-path pass output just before last-state persistence; `RenderStats` mirrors the lighter `last_virtual_geometry_node_and_cluster_cull_instance_seed_count`; and the framework contracts now lock that `query_virtual_geometry_debug_snapshot()` and `query_stats()` agree on the same root-seed worklist scale. That means host tools can inspect the exact root traversal inputs through the same public VG query they already use for `cull_input`, `selected_clusters`, `hardware_rasterization_records`, and `visbuffer64_entries`, without opening another renderer-private seam first.
- The newest follow-up turns those parallel startup records into one explicit compute-stub launch/worklist contract instead of letting later baseline passes reconstruct it from scattered fields. `RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot` now groups `global_state + dispatch_setup + instance_seeds`; `virtual_geometry_node_and_cluster_cull_pass.rs` derives it beside the existing GPU buffers as the authoritative startup package; `seed_backed_execution_selection.rs` now refuses to synthesize execution clusters without that package and reads `cluster_budget`, `forced_mip`, and root seed rows from it instead of reaching back into the pass output piecemeal; and `store_last_runtime_outputs.rs` plus `RenderVirtualGeometryDebugSnapshot.node_and_cluster_cull_launch_worklist` keep the same combined record visible on the public framework query path. The new regressions `node_and_cluster_cull_pass_publishes_launch_worklist_from_global_state_dispatch_and_seeds` and `seed_backed_execution_selection_collection_requires_explicit_launch_worklist_contract`, together with the updated `virtual_geometry_debug_snapshot_contract.rs`, lock that the current baseline path now consumes a real launch package even though hierarchy traversal is still not on GPU yet.
- The newest follow-up materializes that same launch/worklist contract as a renderer-owned GPU buffer instead of leaving it as CPU-only snapshot data. `RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot` now packs to and from a stable `u32` layout; `virtual_geometry_node_and_cluster_cull_pass.rs` allocates `launch_worklist_buffer` from that layout beside the pass-owned DTO; `render.rs`, `render_frame_with_pipeline.rs`, `store_last_runtime_outputs.rs`, `scene_renderer.rs`, `new_with_icon_source.rs`, and `reset_last_runtime_outputs.rs` preserve the buffer on `SceneRenderer`; and `read_node_and_cluster_cull_launch_worklist_snapshot.rs` decodes it back into the combined typed record for tests. This matters because the first real compute-stub consumer can now bind one authoritative startup buffer instead of reassembling `global_state + dispatch_setup + instance_seeds` from three separate renderer seams, while host tooling can still inspect the same combined contract through one readback helper. Focused validation for this slice stayed green on `F:\cargo-targets\zircon-codex-b` with `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_launch_worklist_roundtrips_through_gpu_word_layout --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_node_and_cluster_cull_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_debug_snapshot_contract -- --nocapture`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`.

That keeps the current N3 baseline path honest: even before real `VisBuffer64` and `NodeAndClusterCull` land, the current `ClusterSelection -> baseline raster -> indirect submission -> execution snapshot` bridge no longer throws away per-instance ownership midway through the frame.

It also tightens the N3 debug fallback contract: execution-facing inspection is now resilient when host-built submission mirrors are intentionally dropped for tests, because the GPU authority buffer plus execution indices are sufficient to reconstruct segment ownership, draw-ref lineage, and per-instance execution order from shader-authored truth.

## Automatic Production Extract Synthesis

This slice now also covers the first N2-to-N5 bridge step: automatic synthesis of `RenderVirtualGeometryExtract` from cooked VG assets when the `Virtual Geometry` feature is enabled but the incoming frame extract still carries `geometry.virtual_geometry = None`.

`zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/automatic_extract.rs` adds a deterministic flattening path that:

- walks cooked `VirtualGeometryAsset` payloads attached to model primitives,
- emits every cooked cluster into the production `RenderVirtualGeometryExtract.clusters` list instead of only the CPU-selected frontier,
- remaps local cluster ids and page ids into one global id space across all instances so the current runtime page table and parent-page derivation stay authoritative,
- preserves parent-cluster lineage after remap,
- transforms cluster bounds from local mesh space into world space using the mesh instance transform, and
- seeds initial resident pages from `root_page_table` with deterministic cluster/page budgets derived from the CPU reference plus the cooked root lineage.

`zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/automatic_extract.rs` provides the renderer-side host hook that resolves model assets through the current asset manager and feeds them into the Nanite automatic-extract helper.

The helper input DTO now follows the same owner-boundary rule as the runtime request DTOs. `VirtualGeometryAutomaticExtractInstance` is constructed through `VirtualGeometryAutomaticExtractInstance::new(...)`, so tests and renderer-side synthesis no longer rely on the input field layout when passing entity, optional source model, transform, and cooked `VirtualGeometryAsset` into the flattening path.

The CPU-reference config seam is also constructor-backed: `VirtualGeometryCpuReferenceConfig::new(...)` carries a `VirtualGeometryDebugConfig::new(...)` payload, while sibling modules read debug state through named accessors. The automatic-extract path can still map public `RenderVirtualGeometryDebugState` into CPU-reference config and back, but it no longer depends on the private debug-config field layout.

The CPU-reference traversal output uses the same rule. `VirtualGeometryCpuReferenceFrame`, `VirtualGeometryCpuReferenceNodeVisit`, and `VirtualGeometryCpuReferenceLeafCluster` now keep their traversal vectors and record fields private to `nanite/cpu_reference.rs`; `nanite/automatic_extract.rs` converts them into public render inspection DTOs through named accessors such as `visited_nodes()`, `leaf_clusters()`, `selected_clusters()`, `cluster_id()`, and `loaded()`. That keeps the N2 teaching/debug surface stable while preventing automatic extract or graphics tests from depending on the CPU-reference owner's internal field layout.

That automatic path now returns one internal bundle instead of only the flattened production extract:

- `extract`
  - the current runtime-facing `RenderVirtualGeometryExtract` payload
- `cpu_reference_instances`
  - per-instance CPU-reference BVH inspection for the same cooked asset lineage
- `bvh_visualization_instances`
  - per-instance authored hierarchy visualization rows derived from the same CPU-reference traversal

`VirtualGeometryAutomaticExtractOutput` now keeps those bundle fields private as well. Submit-frame synthesis reads them through `extract()`, `cpu_reference_instances()`, and `bvh_visualization_instances()`, while the test-only resolver can consume the production extract with `into_extract()`. That bundle keeps the production VG path and the teaching/debug BVH path sourced from one cooked-asset walk instead of rebuilding two separate traversals at the render-framework layer or depending on the bundle field names.

That renderer-side bridge now also consumes the prepared-model cache instead of blindly reloading `ModelAsset` from the asset manager every frame. `PreparedModel` retains an `Arc<ModelAsset>` beside the GPU resource, and `ResourceStreamer::load_model_asset(...)` now prefers that cached asset when the prepared revision still matches the current resource revision. If the asset revision changed, it falls back to the asset manager so hot-reimported cooked VG data can still refresh correctly.

The automatic Nanite extract path now also fills the new extract-side metadata:

- each cooked mesh instance contributes one `RenderVirtualGeometryInstance`
- `cluster_offset/page_offset` and `cluster_count/page_count` point into the flattened global `clusters/pages` arrays
- `source_model` is retained when synthesis starts from `RenderMeshSnapshot`
- `mesh_name/source_hint` are copied from cooked VG debug metadata
- extract-level `debug` mirrors the effective CPU-reference debug config, so automatic production extract, CPU-reference inspection, and BVH visualization stay aligned under viewport debug overrides

`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs` now uses that hook before building `VisibilityContext`:

- if the feature is disabled, nothing changes;
- if the caller already authored `geometry.virtual_geometry`, that payload still wins untouched;
- if the feature is enabled and the authored payload is absent, the render framework synthesizes the VG extract from cooked model assets and feeds the supplemented extract into visibility planning and runtime submission.

This keeps the current M5 host/runtime structure intact while letting cooked Nanite-like data participate in the production VG visibility/page workflow.

## Extract Contract Growth

This slice now also upgrades the public `RenderVirtualGeometryExtract` contract so it can carry the first Nanite-style instance/debug metadata without breaking the current `clusters/pages` consumers.

`zircon_runtime/src/core/framework/render/scene_extract.rs` now defines:

- `RenderVirtualGeometryInstance`
  - one synthesized VG instance record with `entity`, optional `source_model`, `transform`, cluster/page range offsets, and optional cooked debug labels
- `RenderVirtualGeometryDebugState`
  - extract-level debug flags matching the current CPU-reference vocabulary: `forced_mip`, `freeze_cull`, `visualize_bvh`, `visualize_visbuffer`, and `print_leaf_clusters`

`RenderVirtualGeometryExtract` now carries both:

- `instances`
- `debug`

The important boundary here is compatibility:

- existing visibility/runtime/prepare code can continue reading only `cluster_budget`, `page_budget`, `clusters`, and `pages`
- new Nanite-oriented code can start consuming instance ranges and debug state from the same extract payload instead of inventing a second side channel

This is the first extract-side convergence step from the M5 plan's "instance-driven input" target. It does not yet replace the current flattened cluster/page bridge.

## Viewport Debug Override And Visibility Consumption

This continuation closes the first gap between the richer extract contract and the production visibility/runtime path.

### Host-Facing Debug Override Plumbing

`zircon_runtime/src/core/framework/render/camera.rs` now lets `SceneViewportExtractRequest` carry `virtual_geometry_debug`.

That debug override is then preserved through:

- `SceneViewportRenderPacket.virtual_geometry_debug`
- `RenderFrameExtract.geometry.virtual_geometry_debug`
- `RenderFrameExtract::to_scene_snapshot()`

`zircon_runtime/src/scene/world/render.rs` copies the request-level override into the viewport render packet, and `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs` applies that override onto the effective VG extract before `VisibilityContext` is built.

This is the first production host path that lets runtime preview and editor viewport drive Nanite-like debug controls without requiring an explicitly authored `geometry.virtual_geometry` payload.

### Instance-Aware Virtual Geometry Planning

`zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs` now treats `RenderVirtualGeometryExtract.instances` as authoritative range metadata instead of ignoring it.

The current behavior is:

- if `instances` is empty, planning falls back to the legacy entity-based scan across `clusters/pages`
- if `instances` is present, planning only considers cluster/page slices covered by the visible instances' `cluster_offset/cluster_count` and `page_offset/page_count`
- `cluster_count` and `cluster_ordinal` now derive from those instance-scoped ranges instead of scanning every cluster that happens to share the entity id

This is still a baseline bridge, not a full Nanite `HierarchyBuffer` traversal. The scope here is narrower: the public extract contract now materially changes runtime planning instead of existing as metadata-only sidecars.

### Debug-State Effects In Visibility

The first two extract-level debug flags now alter production VG planning:

- `forced_mip`
  - filters the eligible cluster set before visibility refinement, so only clusters at the forced mip participate in visibility, page-request, and draw-segment generation
- `freeze_cull`
  - reuses the previous frame's visible cluster ids and requested-page set when history exists, instead of recomputing the cluster frontier from current camera visibility

`visualize_bvh` and `visualize_visbuffer` no longer sit idle at this layer. They now feed the renderer-owned snapshot plus same-frame scene-gizmo overlays through `build_runtime_frame.rs`. `print_leaf_clusters` remains the primary inspection-only debug flag until later GPU debug passes arrive.

## Stats Observability

This continuation also promotes the first Nanite-style `instances/debug` signals into the host-visible stats surface.

`zircon_runtime/src/core/framework/render/backend_types.rs` now exposes these additional `RenderStats` fields:

- `last_virtual_geometry_instance_count`
- `last_virtual_geometry_forced_mip`
- `last_virtual_geometry_freeze_cull`
- `last_virtual_geometry_visualize_bvh`
- `last_virtual_geometry_visualize_visbuffer`
- `last_virtual_geometry_print_leaf_clusters`

`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs` fills those fields from `FrameSubmissionContext.virtual_geometry_extract`, which means the stats surface now reflects the effective authored-or-synthesized VG payload after viewport debug overrides have been applied.

Runtime residency stats now cross the plugin provider boundary instead of a runtime-owned concrete state boundary. `VirtualGeometryRuntimeState` is now the neutral trait from `zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_state.rs`, while the concrete snapshot remains inside `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/`. After record feedback, the provider returns `VirtualGeometryRuntimeStats` with page-table, resident-page, pending-request, completed-page, and replaced-page counts. `record_submission(...)` maps those values into private `VirtualGeometryStatSnapshot` through `VirtualGeometryStatSnapshot::new(...)`, and `update_stats/virtual_geometry_stats.rs` reads page-table/residency/completion plus prepare-owned indirect segment counts through named accessors instead of reaching into submit-record fields. The drawable indirect segment count is still prepared by `VirtualGeometryPrepareFrame::drawable_indirect_segment_count()`, so record/update code does not scan `cluster_draw_segments` or name `VirtualGeometryPrepareClusterState` just to derive the non-missing segment cardinality for public stats. Actual indirect draw count remains a renderer last-output projection because repeated primitive execution can expand one prepare segment into multiple GPU-submitted draws; production stats use `last_virtual_geometry_indirect_draw_count()` for that execution truth, while the segment-count renderer helper stays on the `#[cfg(test)]` inspection side.

`VirtualGeometryRuntimeState.page_budget` has started the same runtime-state owner-boundary convergence that Hybrid GI now uses. Legacy extract registration writes the effective resident-page budget through `set_page_budget(...)`, while completion budget gates, pending-request slot assignment, and prepare-frame available-slot projection read through `page_budget()`. The wider page metadata, residency, pending, evictable, hot-frontier, and slot allocator collections are still being sealed in follow-up slices, but page-budget storage is no longer exposed as a sibling-module field.

Runtime page metadata maps now follow the same seam. Extract registration updates page byte sizes through `insert_page_size(...)` / `retain_page_sizes(...)` and replaces parent topology through `replace_page_parent_pages(...)`; plan ingestion and prepare-frame page export read sizes through `has_page_size(...)` / `page_size_bytes(...)`; lineage-aware pending request ordering, evictable-page ranking, and GPU page-table hot-frontier inheritance read parent links through `page_parent_pages()`. This leaves page metadata storage owner-private while preserving the current Nanite-like parent/descendant scoring behavior.

The current requested-page order map is also owner-private. Visibility plan ingestion resets and seeds it through `clear_current_requested_page_order()` and `ensure_current_requested_page_order(...)`, node-and-cluster-cull feedback appends unseen page requests from `current_requested_page_order_len()`, prepare-frame sorting reads stable ranks through `current_request_rank(...)`, and eviction ranking reads the map through `current_requested_page_order()` when protecting active request lineages. This keeps request-order storage out of sibling modules while preserving existing stable frontier ordering.

Hot-frontier residency caches are now owner-private as well. Feedback refresh decays the cooling window through `retain_recent_hot_resident_pages(...)`, carries current-frame hot pages into the recent cache with `extend_recent_hot_resident_pages(...)`, and replaces the new current set through `replace_current_hot_resident_pages(...)`; completion paths mark inherited hot pages with `insert_current_hot_resident_page(...)` / `extend_current_hot_resident_pages(...)`; eviction removes both current and recent entries through `remove_hot_resident_page(...)`; ranking uses `current_hot_resident_page_ids()` plus `recent_hot_resident_page_ids()` to build the same current+cooling frontier set. The two-frame cooling behavior remains unchanged, but the cache layout no longer leaks across VG runtime sibling modules.

VG slot allocator state has also moved behind runtime-state owner methods. `free_slots` and `next_slot` are private, with residency management using `first_free_slot()`, `remove_free_slot(...)`, `insert_free_slot(...)`, `allocate_next_slot()`, and `advance_next_slot_past(...)`; prepare-frame available slot projection reads `free_slot_ids()` and `next_slot()`; slot assignment validation reads `has_free_slot(...)` plus `next_slot()`. Resident page ownership is still a separate follow-up seam, but the allocator layout no longer crosses module boundaries.

Resident page slot ownership now follows that allocator seam. Prepare-frame resident page export and snapshot/debug helpers read slot pairs through `resident_page_slots()`; page-table reconciliation, stale extract cleanup, feedback cooling, and evictable retain paths read live page ids through `resident_page_ids()`; prepare-visible-cluster state, plan ingestion, pending-request scoring, and tests query `resident_slot(...)` / `has_resident_page(...)`; residency mutation paths use `insert_resident_page_slot(...)` / `remove_resident_page_slot(...)`. This keeps the page-to-slot map private while preserving the existing page-table and resident fallback behavior.

The runtime pending-upload set/vector now uses the same owner API. Visibility plan ingestion and node-and-cluster-cull feedback queue work through `insert_pending_page(...)` plus `push_pending_page_request(...)`; extract cleanup, test fulfillment, and promotion cleanup use `retain_pending_pages(...)`, `remove_pending_page(...)`, and `retain_pending_page_requests(...)`; prepare-frame request sorting reads `pending_page_requests()`, completion gates use `has_pending_page(...)`, and snapshot/test projection reads counts or ids through `pending_request_count()` and `pending_page_id_iter()`. `VirtualGeometryPageRequest` still owns the per-request DTO fields, while the queue layout no longer leaks from `VirtualGeometryRuntimeState`.

The evictable page queue closes the remaining public `VirtualGeometryRuntimeState` field seam. Extract registration clears through `clear_evictable_pages()`, visibility plan ingestion replaces the queue through `replace_evictable_pages(...)`, prepare-frame and test projections read with `evictable_page_ids()`, eviction removes a page with `remove_evictable_page(...)`, and completion/page-table reconciliation keeps the queue resident-only through `retain_resident_evictable_pages()`. After this slice the VG runtime-state declaration is fully owner-private and sibling modules coordinate through methods instead of raw collection fields.

The runtime pending-upload queue has the same owner rule. `VirtualGeometryPageRequest` is created only by the VG runtime ingestion path and exposes page id, byte size, and generation as named accessors. Prepare-frame assembly and graphics regressions now project through those accessors instead of constructing or reading raw request fields, keeping the upload request DTO movable with the runtime host.

The runtime feedback handoff now follows the same direction for page-upload pressure, but the owner is the linked plugin provider rather than `zircon_runtime` concrete state. `VirtualGeometryRuntimeFeedback` carries the optional renderer GPU completion, visibility feedback, node-and-cluster-cull page-request feedback, and the evictable page ids required when completion applies replacements. `collect_runtime_feedback(...)` converts renderer-private readback completion parts into neutral `VirtualGeometryGpuCompletion`; `record_submission(...)` attaches prepared evictable pages through `PreparedRuntimeSubmission::take_virtual_geometry_evictable_page_ids()` and `with_evictable_page_ids(...)`; `VirtualGeometryRuntimeState::update_after_render(...)` then consumes only `gpu_completion()`, `visibility_feedback()`, `node_and_cluster_cull_page_requests()`, and `evictable_page_ids()` from the neutral DTO. Visibility-plan/hot-frontier refresh, completion apply, page-request ingest, and replacement pressure therefore cross the record boundary through one erased VG runtime owner instead of loose `Vec<u32>` parameters, direct prepared-submission field reads, `FrameSubmissionContext.virtual_geometry_feedback` reads inside the update path, or concrete plugin state names inside `zircon_runtime`.

The viewport-record scratch state that seeds this frame context now follows the same owner rule. `ViewportRecordState` keeps previous visibility, previous Virtual Geometry runtime state, pipeline asset/options, capability summary, and predicted generation behind constructor/accessor/take methods, so `build_frame_submission_context(...)` no longer reads scratch fields directly while assembling the effective VG extract, visibility context, and previous runtime handoff.

This matters for the Nanite convergence plan because the richer extract contract is no longer observable only through internal visibility behavior. Runtime hosts can now query whether a frame actually ran with:

- instance-driven VG input,
- a forced mip override,
- frozen culling,
- BVH/VisBuffer debug visualization intent, or
- leaf-cluster print mode.

The same stats updater also clears those fields when the next frame no longer carries an effective VG payload, so host-side tooling does not keep stale Nanite debug state alive across non-VG frames.

## Renderer Inspection Snapshot

This continuation also adds a renderer-owned inspection surface for the richer Nanite-style debug state instead of pushing more transient fields into `RenderStats`.

`zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs` defines `RenderVirtualGeometryDebugSnapshot`, and `RenderFramework::query_virtual_geometry_debug_snapshot()` now exposes it through the public framework API.

The snapshot is assembled in `build_virtual_geometry_debug_snapshot.rs` from the effective `FrameSubmissionContext` state after authored data, automatic cooked-VG synthesis, viewport debug overrides, visibility feedback, and page-upload planning have all been applied. That keeps the query aligned with what the renderer actually used for the most recent frame instead of only reflecting author intent.

The current snapshot surface is intentionally host-oriented and minimal:

- `instances`
- `debug`
- `cpu_reference_instances`
  - one entry per automatically synthesized cooked-VG instance
  - each entry carries `instance_index`, `entity`, `mesh_name`, `source_hint`
  - `visited_nodes` records BVH node id/depth/page/mip/leaf state plus the node-local cluster ids
  - `leaf_clusters` records leaf-node cluster membership, loaded state, parent cluster id, and bounds/error metadata
  - `page_cluster_map` records the asset-local page-to-cluster mapping
  - `depth_cluster_map` records cluster ids grouped by BVH depth, preserving the CPU-reference traversal order across siblings within the same layer
  - `mip_cluster_map` records leaf-cluster ids grouped by mip level so hosts can dump mip distributions and filter a specific mip directly
  - ids remain asset-local inside this inspection surface even though the production extract remaps cluster/page ids into one global runtime id space
- `bvh_visualization_instances`
  - populated only when `debug.visualize_bvh` is enabled
  - one entry per automatically synthesized cooked-VG instance
  - each entry carries a ready-to-draw BVH node tree with `parent_node_id`, `child_node_ids`, node depth, page/mip ownership, direct node-local `cluster_ids`, subtree-selected/resident cluster ids, and node bounds/error metadata
  - this keeps BVH visualization on the same renderer-owned inspection path instead of introducing a separate Nanite BVH query
- `visible_cluster_ids`
- `selected_clusters`
  - one entry per current-frame cluster selected by the prepare-owned baseline path at submission-build time, then re-authoritatively filtered to the real execution subset when the renderer stores last-state
  - each entry carries `instance_index`, `entity`, `cluster_id`, `cluster_ordinal`, `page_id`, `lod_level`, and the current resident/pending/missing execution state represented by the active snapshot phase
  - this gives hosts and future raster work one stable current-frame cluster worklist without reverse-engineering it from visbuffer color tags
- `visbuffer_debug_marks`
  - populated only when `debug.visualize_visbuffer` is enabled
  - now derived from `selected_clusters` during submission-build snapshot assembly when prepare-owned cluster selection truth exists
  - same-frame overlay construction first gates on `debug.visualize_visbuffer`, then prefers prepare-time `unified_indirect_draws()` / non-missing visible-cluster truth so authored-VG and automatic cooked-VG frames follow the same current-frame worklist before post-render execution backfill
  - when prepare-derived marks are unavailable, the current-frame overlay path can still fall back to the submission snapshot marks as a compatibility seam
  - the renderer-owned stored snapshot is then backfilled from actual `execution_segments` during `store_last_runtime_outputs.rs`, so missing visibility-only clusters no longer survive into the post-render inspection surface
  - each stored mark carries `instance_index`, `entity`, `cluster_id`, `page_id`, `lod_level`, execution-derived resident/pending state, and a deterministic RGBA debug tag
  - this remains explicit compatibility-path inspection truth, not a claim that real `VisBuffer64` pixels already exist
- `visbuffer64_clear_value`
  - currently published as `0`
  - defines the clear contract for the first renderer-owned `VisBuffer64` abstraction before real pixel storage exists
- `visbuffer64_entries`
  - always derived from the same `selected_clusters` worklist that drives post-render execution inspection, not gated by `debug.visualize_visbuffer`
  - each entry carries `entry_index`, `packed_value`, and the decoded `instance_index/entity/cluster_id/page_id/lod_level/state` metadata that produced that 64-bit word
  - the current baselineibility pack layout uses fixed-width fields for `cluster_id`, `page_id`, `instance_index`, `lod_level`, and execution state so hosts can inspect stable 64-bit visibility results without opening another query path
  - this is still a logical visibility-entry stream, not a claim that the engine already owns a pixel-addressable Nanite `VisBuffer64` texture
- `requested_pages`
- `resident_pages`
- `dirty_requested_pages`
- `evictable_pages`
- prepare-backed page residency/request inspection:
  - `resident_page_inspections`
  - `pending_page_request_inspections`
  - `available_page_slots`
  - `evictable_page_inspections`
  - each resident/evictable page inspection carries `page_id`, `slot`, and `size_bytes`
  - each pending request inspection carries `page_id`, `size_bytes`, `generation`, `frontier_rank`, `assigned_slot`, and `recycled_page_id`
- `leaf_clusters`, but only when `print_leaf_clusters` is enabled
- render-derived execution summary:
  - `execution_segment_count`
  - `execution_page_count`
  - `execution_resident_segment_count`
  - `execution_pending_segment_count`
  - `execution_missing_segment_count`
  - `execution_repeated_draw_count`
  - `execution_indirect_offsets`
- render-derived execution segment view:
  - `execution_segments`
  - each `execution_segment` carries `entity`, `page_id`, `draw_ref_index`, best-effort submission token data, cluster ordinal/span/total counts, submission slot, execution state, lineage depth, lod level, frontier rank, and `original_index`
- render-derived submission truth:
  - `submission_order`
  - `submission_records`
  - each `submission_record` carries `entity`, `page_id`, best-effort `draw_ref_index`, `submission_index`, `draw_ref_rank`, and `original_index`

Renderer ownership is important here. `ViewportRenderFrame` now carries an internal `virtual_geometry_debug_snapshot`, `SceneRenderer` stores the last submitted copy inside its Virtual Geometry last-state, and `WgpuRenderFramework` returns that renderer-owned snapshot through the query API. That means runtime preview and editor hosts can inspect BVH/leaf-cluster intent without coupling themselves to `RenderStats` reset policy or to future GPU readback layout changes.

The execution-summary part of the snapshot is intentionally filled in two stages:

- `build_virtual_geometry_debug_snapshot.rs` still constructs the host-facing snapshot from the effective `FrameSubmissionContext`, preserving authored-or-synthesized extract state, visibility feedback, and page-upload outcomes before rendering starts.
- the same submission context now also carries the automatic cooked-VG `cpu_reference_instances` bundle, so `RenderFramework::query_virtual_geometry_debug_snapshot()` can expose per-instance BVH node visits, leaf clusters, and page maps without adding a second Virtual Geometry query API.
- that same builder now also turns `visualize_bvh` into concrete `bvh_visualization_instances` and turns `visualize_visbuffer` into concrete `visbuffer_debug_marks`, so those debug flags are no longer host-visible intent only.
- when prepare-time unified-draw truth exists, `build_virtual_geometry_debug_snapshot.rs` now seeds `visbuffer_debug_marks` from `VirtualGeometryPrepareFrame::same_frame_visbuffer_debug_marks(...)` instead of re-expanding the broader visibility frontier, so the submission-build snapshot and the same-frame overlay now share one authoritative compatibility-path source before render-time execution backfill happens.
- `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_cluster_selection.rs` now defines `VirtualGeometryClusterSelection`, and `VirtualGeometryPrepareFrame::cluster_selections(...)` exposes the prepare-owned current-frame cluster worklist before any visbuffer/debug or raster projection happens.
- `VirtualGeometryPrepareFrame::selected_clusters(...)`, `same_frame_visbuffer_debug_marks(...)`, and `cluster_raster_draws(...)` are now all derived views over that single `cluster_selections(...)` result instead of maintaining parallel cluster-remap or raster-remap logic.
- `build_virtual_geometry_debug_snapshot.rs` now publishes the projected public `RenderVirtualGeometrySelectedCluster` records from that same internal worklist when prepare truth exists, and its submission-build `visbuffer_debug_marks` are derived from the same selected-cluster list.
- `build_runtime_frame.rs` now re-derives same-frame visbuffer marks from the prepare-time unified draw list when that truth exists, and it keeps the overlay disabled when `visualize_visbuffer` is false so baseline frames do not accidentally inherit the baselineibility marker path.
- `ViewportRenderFrame` now also carries `virtual_geometry_cluster_selections`, which is populated from `VirtualGeometryPrepareFrame::cluster_selections(...)` during runtime-frame assembly instead of snapshotting a pre-expanded raster-only map.
- `VirtualGeometryClusterRasterDraw` remains the fixed-fanout fallback raster DTO, but it is now a projection from `VirtualGeometryClusterSelection` instead of a separately-owned runtime-frame seam.
- `store_last_runtime_outputs.rs` then backfills the actual render-derived execution summary, indirect offsets, and execution submission order/records into that same snapshot just before it is stored as renderer last-state.
- that same store step now also rebuilds `selected_clusters` from the actual `execution_segments` plus entity-local cluster ranges, so the stored public cluster worklist shrinks from the broader submission-build compatibility set down to the real raster-submitted subset whenever execution filtering removed clusters.
- `visbuffer_debug_marks` are then re-derived from that stored execution-backed `selected_clusters` list, so the post-render inspection surface keeps one authoritative cluster-selection truth instead of parallel backfill paths for worklists and color marks.
- when runtime submission token records and draw-ref records are both present, that same store step also merges them by `original_index` so `submission_records` can expose `draw_ref_index` without a second public query surface.
- `virtual_geometry_indirect_stats.rs` now also converts the filtered `execution_draws` list into typed `execution_segments` before `render_frame_with_pipeline.rs` stores the renderer-owned snapshot, so host tooling can inspect execution state/LOD/submission-slot data without going through test-only GPU readback helpers.
- those typed `execution_segments` now also retain `instance_index`, so post-render inspection and later raster work can keep instance ownership without reopening entity-local cluster scans.

That split keeps one query surface while avoiding a parallel API just to expose `SceneRenderer`'s post-render Virtual Geometry execution counters.

## Runtime BVH / VisBuffer Overlay Rendering

This continuation upgrades `visualize_bvh` and `visualize_visbuffer` from "host can inspect debug data through the snapshot" to "the renderer actually draws debug overlays through the normal overlay pass stack."

`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs` is now the convergence point:

- it still builds the renderer-owned debug snapshot from `FrameSubmissionContext`
- it now also converts `FrameSubmissionContext.virtual_geometry_bvh_visualization_instances` into `RenderOverlayExtract.scene_gizmos` before `ViewportRenderFrame::from_extract(...)` snapshots the scene
- it also converts current-frame visbuffer marks back into per-cluster scene-gizmo markers using the effective production cluster bounds from `FrameSubmissionContext.virtual_geometry_extract`
- it now also snapshots the current frame's unified internal cluster-selection worklist into `ViewportRenderFrame.virtual_geometry_cluster_selections` by calling `VirtualGeometryPrepareFrame::cluster_selections(...)`, creating a runtime-frame-owned work submission seam beside the debug snapshot seam
- `VirtualGeometryPrepareFrame::same_frame_visbuffer_debug_marks(...)` now owns the authoritative same-frame visbuffer mark derivation from `unified_indirect_draws()` plus visible-cluster fallback, so `build_runtime_frame.rs` no longer reconstructs cluster ordering, state mapping, or instance lookup on its own
- the same-frame visbuffer path now respects the `visualize_visbuffer` gate, prefers prepare-owned current-frame truth over stale submission-time snapshot marks, and still preserves a compatibility fallback when prepare cannot contribute marks
- host-authored overlays are preserved; the Nanite-like debug overlays are appended instead of replacing existing `scene_gizmos`

That same runtime-frame seam now also feeds the teaching fallback raster path one layer later:

- `build_mesh_draw_build_context.rs` still prefers `prepare.visible_entities` when prepare is present, but it now falls back to the entities present in `ViewportRenderFrame.virtual_geometry_cluster_selections` when prepare is absent
- `build_virtual_geometry_cluster_raster_draws.rs` now prefers the frame-owned selection seam and projects baseline raster draws from it, only falling back to recomputing from `virtual_geometry_prepare` when the new field is absent
- this keeps the current teaching raster path behavior stable while moving the actual runtime ownership boundary onto `ClusterSelection`, which is the intended bridge toward a future dedicated `ClusterSelection -> HardwareRasterization` handoff

The actual drawable representation stays deliberately simple and deterministic for this teaching/debug phase:

- each BVH node becomes an AABB-style wireframe built from the node's `bounds_center` and `bounds_radius`
- parent-child BVH relationships become connector lines between node centers
- each visible-cluster visbuffer mark becomes a lifted leader-line plus cross/wireframe marker anchored to the production cluster bounds so it survives the shared depth-tested gizmo pass on the same frame
- line colors encode the current Nanite-like state already exposed by the snapshot:
  - unselected/internal traversal context
  - selected and fully resident subtree
  - selected but partially resident subtree
  - selected but non-resident subtree
  - visbuffer mark colors still come from the deterministic per-cluster RGBA tags published by the snapshot

`SceneGizmoKind` now includes `VirtualGeometryBvh` and `VirtualGeometryVisBuffer`, but the implementation intentionally uses line-only gizmos with no icon dependency. That keeps the feature inside the existing overlay renderer instead of adding a parallel BVH/visbuffer debug pass or a second render-only debug surface.

This is still not a real `VisBuffer64` texture debug view. The important convergence point is architectural: the renderer now owns both a logical 64-bit visibility-entry stream (`visbuffer64_entries`) and the current overlay/debug compatibility view, and both are sourced from the same `ClusterSelection -> selected_clusters -> execution subset` seam through the existing SRP/runtime-frame submission path.

## Asset Data Model

`zircon_runtime/src/asset/assets/model.rs` now defines a Nanite-like cooked asset payload:

- `VirtualGeometryHierarchyNodeAsset`
  - One hierarchy node with parent link, up to four child node ids, cluster range, owning page id, mip level, and bounds/error metadata.
- `VirtualGeometryClusterHeaderAsset`
  - One cluster record with page ownership, hierarchy node ownership, LOD/mip level, parent cluster link, and the bounds/error fields needed by later culling logic.
- `VirtualGeometryClusterPageHeaderAsset`
  - Page id, byte offset, and byte size for one cluster page.
- `VirtualGeometryRootClusterRangeAsset`
  - Root-facing cluster range metadata used by the CPU reference traversal to seed the hierarchy walk deterministically.
- `VirtualGeometryDebugMetadataAsset`
  - Human-oriented labels and notes for dumps, inspection, and teaching content.
- `VirtualGeometryAsset`
  - The cooked container that groups the buffers above plus `cluster_page_data` and `root_page_table`.

`ModelPrimitiveAsset.virtual_geometry` is optional and defaults to `None`. This keeps existing mesh-only assets valid while allowing a single primitive to carry both:

- legacy triangle data for compatibility fallback, and
- a cooked Nanite-like hierarchy/page payload for the new VG path.

The importer and builtin model constructors now initialize `virtual_geometry: None`, so existing asset ingestion remains behaviorally stable until a real VG cook step populates the field.

## Project Asset Ingestion

This slice also closes the first real project-asset gap for cooked VG data: `.model.toml` files are now first-class source assets in the project importer.

`zircon_runtime/src/asset/importer/ingest/import_model.rs` parses a source `.model.toml` file directly into `ModelAsset`, and `import_from_source.rs` now routes `*.model.toml` through that path before the generic extension dispatch.

That matters because the Nanite automatic-extract bridge already resolves `ModelAsset` payloads from the project asset manager. Without a source importer for `.model.toml`, cooked VG payloads only existed as schema and tests, not as stable project content. With this step in place:

- a project can author or cook `res://models/*.model.toml` files that carry `virtual_geometry`,
- the normal asset pipeline imports them as `ResourceKind::Model`,
- `ProjectAssetManager::load_model_asset(...)` returns the cooked VG payload intact, and
- the existing Nanite fallback synthesis path can consume those project assets without introducing a parallel asset entry path.

## CPU Reference Flow

`zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/cpu_reference.rs` defines the teaching/reference path.

### Inputs

- `VirtualGeometryAsset`
- an entity id that will own the generated extract clusters
- a resident page set
- `VirtualGeometryCpuReferenceConfig`, currently centered on `VirtualGeometryDebugConfig`

### Debug Controls

The first debug surface matches the plan vocabulary even though only part of it is active today:

- `forced_mip`
- `freeze_cull`
- `visualize_bvh`
- `visualize_visbuffer`
- `print_leaf_clusters`

For this slice, `forced_mip` is the active selector. The other flags are stored as stable API surface for later passes.

### Traversal Behavior

`VirtualGeometryCpuReferenceFrame::from_asset(...)` performs:

1. Root discovery from `root_cluster_ranges` or parentless hierarchy nodes.
2. Deterministic depth-first hierarchy traversal.
3. Per-node visit recording:
   - node id
   - depth
   - page id
   - mip level
   - leaf/non-leaf state
   - cluster ids covered by the node range
4. Leaf cluster recording with:
   - node ownership
   - cluster id
   - page id
   - mip level
   - resident/non-resident status
   - parent cluster link
   - bounds/error metadata
5. Page-to-cluster mapping for debug and future residency/page tooling.

The traversal now makes the Nanite plan vocabulary explicit inside the CPU reference path:

- `visit_node(...)`
  - records one `VirtualGeometryCpuReferenceNodeVisit` with node id, depth, page id, mip level, leaf state, and the node-local cluster ids
  - returns whether the hierarchy node is a leaf so the caller can decide between descent and cluster storage
- `store_cluster(...)`
  - records every encountered leaf cluster into `leaf_clusters`
  - promotes only resident, mip-accepted leaf clusters into `selected_clusters`

This keeps the single-threaded teaching path aligned with the later N4 `VisitNode / StoreCluster` cull vocabulary before `NodeAndClusterCullPass` exists.

Selected clusters are currently defined as:

- resident page only, and
- mip matches `forced_mip` when that override is present.

That rule is deliberately simple. It gives Zircon a deterministic golden reference before automatic SSE-driven LOD and multi-pass BVH culling are introduced.

The CPU reference bridge now also emits one `RenderVirtualGeometryInstance` plus `RenderVirtualGeometryDebugState` when it converts into `RenderVirtualGeometryExtract`, so the teaching/reference path and the production automatic path both feed the same richer extract contract.

## Bridge To Current Virtual Geometry

`VirtualGeometryCpuReferenceFrame::to_render_extract(...)` is the baseline bridge from the new Nanite-like data model to the current M5 VG surface.

It produces:

- `RenderVirtualGeometryCluster` entries from the CPU-selected leaf clusters
- `RenderVirtualGeometryPage` entries from the cooked page headers plus the supplied resident-page set
- the existing `cluster_budget` / `page_budget` fields expected by the current visibility/runtime pipeline

This is the key “gradual absorption” step:

- the cooked Nanite-style asset and hierarchy logic exist now,
- but they still flow into the existing `RenderVirtualGeometryExtract` contract,
- so the current M5 runtime can absorb the new data incrementally instead of being replaced wholesale.

## Execution Modes

`zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/execution_mode.rs` defines the first execution-mode contract:

- `FlagshipGpu` when `RenderCapabilitySummary.virtual_geometry_supported` is true
- `BaselineGpu` when the backend cannot claim flagship VG support but still exposes a usable render surface/offscreen path
- `CpuDebug` when no GPU-backed VG path should be assumed

This is only routing policy for now. The actual runtime still needs later work to switch behavior between these modes.

## Validation

This slice is locked by two focused tests:

- `zircon_runtime/src/asset/tests/assets/model.rs`
  - proves `ModelAsset` round-trips a cooked `virtual_geometry` payload through TOML
- `zircon_runtime/src/asset/tests/pipeline/manager.rs`
  - proves a source `res://models/*.model.toml` file imports through `ProjectAssetManager` as a model resource and preserves the cooked VG payload end-to-end
- `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs`
  - proves current prepared model assets short-circuit fallback loading and stale prepared assets fall back to the latest asset load
- `zircon_runtime/tests/virtual_geometry_extract_contract.rs`
  - proves the public `RenderVirtualGeometryExtract` contract now carries instance ranges plus debug state and preserves them through clone/equality semantics
- `zircon_runtime/tests/virtual_geometry_visibility_debug_contract.rs`
  - proves viewport-request VG debug overrides round-trip into `RenderFrameExtract`
  - proves visibility planning now respects instance cluster/page ranges
  - proves `forced_mip` filters the production VG visibility set
  - proves `freeze_cull` preserves previous visible clusters and requested pages through history
- `zircon_runtime/tests/virtual_geometry_stats_contract.rs`
  - proves render-framework stats expose effective VG instance/debug state
  - proves render-framework stats now also mirror the authored VG cull-input scale (`cluster/page budget`, authored `cluster/page` counts, and `visible_entity_count`) from the same renderer-owned snapshot that will later align with `NaniteGlobalStateBuffer`
  - proves render-framework stats now also mirror the first pass-owned `NodeAndClusterCull` startup provenance/count (`last_virtual_geometry_node_and_cluster_cull_source` and `last_virtual_geometry_node_and_cluster_cull_record_count`) and clear them back to `Unavailable` / `0` on a later non-VG frame
  - proves render-framework stats expose the execution-backed `selected_cluster_count` from the same renderer-owned snapshot that publishes `selected_clusters`
  - proves render-framework stats also mirror the selected-cluster render-path provenance from the same renderer-owned snapshot and clear it back to `Unavailable` on a later non-VG frame
  - proves those stats clear back to defaults once the effective VG payload disappears
- `zircon_runtime/src/graphics/tests/virtual_geometry_execution_stats.rs`
  - proves the public selected-cluster count stays execution-compacted and does not expand back to the repeated indirect-draw workload
- `zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs`
  - proves the new renderer-owned cull-input buffer exists and decodes correctly even on direct renderer paths with no framework snapshot and no uploader readback
  - proves the first pass-owned `NodeAndClusterCull` startup buffer decodes back to the same packed cull-input DTO as the renderer-owned cull-input seam
  - proves the widened `NodeAndClusterCull` startup buffer also round-trips viewport size, camera translation, and the typed view-projection matrix through the renderer-owned global-state helper
  - proves empty-VG frames still publish one zero-work startup record for the future `NaniteGlobalStateBuffer` consumer even while downstream selection/raster passes stay clear-only
- `zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs`
  - proves the descriptor-linked framework exposes the renderer-owned VG debug snapshot
  - proves the public VG debug snapshot now also exposes the first-pass `NodeAndClusterCull` startup provenance/count and agrees with `RenderStats`
  - proves the same public snapshot now also exposes the typed `NodeAndClusterCull` global-state record and keeps its embedded cull-input provenance aligned with the final frame-owned source
  - proves visible-cluster ids, page residency/request state, and optional leaf-cluster output reflect the effective frame submission
  - proves the same snapshot now exposes `selected_clusters`, so host tooling can inspect the current-frame cluster worklist directly instead of reverse-engineering it from `visbuffer_debug_marks`
  - proves automatic cooked-VG synthesis also exposes per-instance CPU-reference BVH node visits, leaf clusters, and page-to-cluster maps through that same snapshot
  - proves `visualize_bvh` now exposes concrete `bvh_visualization_instances` for automatic cooked-VG assets through that same snapshot
  - proves `visualize_bvh` now also changes the captured frame through the shared overlay renderer, so BVH visualization is no longer snapshot-only
  - proves `visualize_visbuffer` now exposes concrete `visbuffer_debug_marks` for the current visible production cluster set through that same snapshot
  - proves the stored snapshot now re-filters those `visbuffer_debug_marks` through actual `execution_segments`, so missing visibility-only clusters are removed from the post-render inspection surface
  - proves `visualize_visbuffer` now also changes the captured frame through the shared overlay renderer for automatic cooked-VG content, so the current visbuffer compatibility view is no longer snapshot-only
  - proves the same snapshot now exposes prepare-backed resident slot mapping, pending request metadata, and available-slot truth without adding a second VG inspection API
  - proves render-derived execution summary counts and indirect offsets are backfilled into the same snapshot and stay aligned with `RenderStats`
  - proves typed `execution_segments` are queryable from that same snapshot and preserve resident/pending execution state plus execution-owned `instance_index`
  - proves render-derived submission order/records are queryable from that same snapshot without opening a second VG inspection API
  - proves the same snapshot now exposes `selected_clusters_source`, so post-render selected-cluster provenance is inspectable on the same surface as `selected_clusters`, `visbuffer64_source`, and `hardware_rasterization_source`
  - proves both `submission_order` and `submission_records` now preserve execution-owned `instance_index`, so host tooling can inspect per-instance submission order without rejoining against `execution_segments`
  - proves `submission_records` can carry `draw_ref_index` when the execution submission and draw-ref channels are both available
  - proves a non-VG frame clears the last snapshot back to `None`
- `zircon_runtime/tests/virtual_geometry_execution_snapshot_contract.rs`
  - proves stored `execution_segments` now keep `instance_index` from the authoritative cluster-selection/submission seam instead of forcing later consumers to reconstruct instance ownership from the extract
  - proves the stored snapshot re-filters `selected_clusters` through actual `execution_segments`, so post-render consumers observe the same authoritative execution-backed worklist that the renderer submitted
  - proves the same store-time authoritative rebuild keeps `visbuffer_debug_marks` aligned with that execution-backed cluster subset instead of preserving missing visibility-only clusters
  - proves the same execution-backed subset now also emits stable `visbuffer64_entries` plus a published clear value, so the first `VisBuffer64` abstraction is sourced from the same authoritative post-render worklist as the rest of the execution-facing snapshot
- `zircon_runtime/tests/virtual_geometry_visbuffer_overlay_contract.rs`
  - proves same-frame `visualize_visbuffer` overlays follow the real non-missing execution subset for an explicit authored VG extract instead of resurrecting missing clusters
- `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs`
  - proves the prepare layer itself exposes same-frame visbuffer marks derived from unified draw truth before the renderer-owned snapshot is backfilled from execution
- `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/types/virtual_geometry_prepare/frame.rs`
  - also proves the prepare layer exposes `selected_clusters(...)` as a prepare-owned cluster worklist derived from unified draw truth before marks are projected from it
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs`
  - proves the submission-build snapshot now prefers prepare-owned same-frame visbuffer marks when prepare has already projected the authoritative draw subset
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs`
  - proves the runtime-frame overlay path consumes prepare-owned same-frame visbuffer marks and still follows the prepare-time unified draw fallback when the stored snapshot is still empty
  - proves the same module-local reconstruction stays disabled when `visualize_visbuffer` is false
- `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs`
  - proves mesh-build assembly now accepts frame-owned VG cluster raster input without direct prepare access, preserving allowed-entity gating and per-entity draw ownership from the new runtime-frame seam
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs`
  - proves the stored renderer-owned snapshot re-authoritatively rebuilds `selected_clusters` from `execution_segments` when submission-build selection was broader than the real execution subset
- `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs`
  - proves the shared indirect-args layout preserves `instance_index` inside renderer-facing `VirtualGeometrySubmissionDetail`, so execution ownership stays sourced from the same authoritative submission plan
- `zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs`
  - proves execution segments still survive when the shared indirect segment and draw-ref buffers are dropped, as long as execution indices and GPU authority remain available
  - proves execution segments and submission records can reconstruct the same per-instance ownership from `execution indices + GPU authority` and can still recover `draw_ref_index` when the host-built execution-record mirror is gone
  - proves the helper submission-order and helper submission-record surfaces now preserve `instance_index` instead of collapsing back to entity/page-only tuples once execution-owned truth is available
- `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_nanite_cpu.rs`
  - proves execution-mode selection
  - proves hierarchy traversal, page mapping, and `forced_mip` filtering
  - proves the CPU reference `visit_node(...)` helper records node/depth/page/mip/cluster-id visit semantics
  - proves the CPU reference `store_cluster(...)` helper stores every leaf while only selecting resident, mip-accepted clusters
  - proves the bridge into `RenderVirtualGeometryExtract`
  - proves automatic extract synthesis remaps multi-instance cluster/page ids into a global space
  - proves world-space bounds and parent-cluster lineage survive the remap
  - proves mesh-snapshot/model-resolver synthesis only collects cooked models
  - proves mesh-based automatic extract keeps the public `RenderVirtualGeometryExtract.debug` state aligned with the same debug override that already drives CPU-reference/BVH synthesis
  - proves CPU-reference inspection groups cluster ids by BVH depth for direct per-layer Nanite dumps
  - proves CPU-reference inspection groups leaf clusters by mip level for direct mip distribution dumps and `Mip=10` filtering
  - proves explicit authored VG payload still overrides the automatic fallback

Focused validation completed for this slice:

- `cargo test -p zircon_runtime --locked asset::tests::pipeline::manager::asset_manager_imports_model_toml_with_virtual_geometry_payload -- --exact --nocapture`
- `cargo test -p zircon_runtime --locked asset::tests::assets::model::model_asset_toml_roundtrip_preserves_virtual_geometry_payload -- --exact --nocapture`
- `cargo test -p zircon_runtime --locked prepare_frame_exposes_same_frame_visbuffer_marks_from_unified_draw_truth --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked prepare_frame_exposes_cluster_selection_from_unified_draw_truth --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked debug_snapshot_prefers_prepare_owned_same_frame_visbuffer_marks_when_available --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked same_frame_visbuffer_marks_ --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked shared_indirect_args_layout_preserves_instance_index_in_submission_details --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_ --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_stats_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_debug_snapshot_contract -- --nocapture`
- `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`
- `cargo test -p zircon_runtime --locked virtual_geometry_shared_indirect_segments_preserve_instance_index_for_submission_fallback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_execution_segments_survive_without_shared_segment_and_draw_ref_buffers --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_execution_segments_survive_with_execution_indices_and_gpu_authority_buffer_only --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_submission_records_survive_with_execution --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_submission_records_survive_with_execution_indices_and_gpu_authority_buffer_only --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_submission_records_survive_with_execution_authority_buffer_only --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked build_context_accepts_frame_owned_virtual_geometry_cluster_selections_without_prepare --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked rebuild_selected_clusters_from_execution_segments_drops_visibility_only_superset --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_renderer_mesh_draw_submission_order_tracks_visibility_owned_unified_indirect_authority --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked visbuffer64_pass_words_follow_executed_submission_keys_and_deduplicate_clusters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b resolve_selected_clusters_for_store_prefers_pass_owned_selected_clusters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b hardware_rasterization_pass_records_follow_executed_submission_keys_and_preserve_startup_parameters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b executed_cluster_selection_pass_filters_deduplicates_and_sorts_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b visbuffer64_pass_entries_follow_shared_executed_cluster_selection_order --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b visbuffer64_pass_prefers_pass_owned_selected_clusters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b visbuffer64_pass_packs_words_from_pass_owned_entries --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b hardware_rasterization_pass_records_follow_shared_executed_cluster_selection_order_and_preserve_startup_parameters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b hardware_rasterization_pass_prefers_pass_owned_selected_clusters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b resolve_visbuffer64_entries_for_store_prefers_pass_owned_entries --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b resolve_visbuffer64_entries_for_store_rebuilds_when_pass_entries_are_missing --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_execution_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_gpu_readback_exposes_execution_backed_visbuffer64_entries --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_gpu_readback_exposes_execution_backed_visbuffer64_entries --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_execution_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b render_framework_automatic_virtual_geometry_bvh_selected_clusters_follow_forced_mip_override --test virtual_geometry_debug_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_debug_snapshot_contract -- --nocapture`
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline virtual_geometry_nanite_mesh_based_automatic_extract_with_debug_keeps_extract_debug_in_sync -- --nocapture`
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline virtual_geometry_nanite_cpu_reference_instances_expose_clusters_grouped_by_bvh_depth -- --nocapture`
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline virtual_geometry_nanite_cpu_reference_instances_expose_leaf_clusters_grouped_by_mip -- --nocapture`
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline visit_node_records_visit_order_and_cluster_ids -- --nocapture`
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline store_cluster_keeps_all_leafs_and_selects_only_resident_matching_mip -- --nocapture`
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline virtual_geometry_nanite_ -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_extract_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_execution_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --test virtual_geometry_visbuffer_overlay_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_visibility_debug_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_stats_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b render_framework_stats_expose_virtual_geometry_instance_ranges_and_debug_state --test virtual_geometry_stats_contract -- --nocapture --exact`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b render_framework_stats_expose_actual_virtual_geometry_execution_compaction --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_execution_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --lib render_framework_ -- --nocapture`
- `cargo check -p zircon_runtime --locked --lib`
- `cargo test -p zircon_runtime --locked --lib scene_prepare_card_capture_resource_snapshot -- --nocapture`
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline virtual_geometry_nanite_ -- --nocapture`
- `cargo test --workspace --locked --target-dir F:\cargo-targets\zircon-codex-a`

2026-04-29 workspace follow-up refreshed the descriptor-linked integration setup after the base renderer stopped implicitly carrying advanced VG descriptors: `cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --nocapture` passed 6/6, and `cargo test -p zircon_runtime --test virtual_geometry_stats_contract --test virtual_geometry_visbuffer_overlay_contract --test virtual_geometry_execution_snapshot_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --nocapture` passed 4/4 across the three related integration targets.

The latest `VisBuffer64` provenance follow-up also locks the zero-selection clear path: when the baseline VG pass executes but emits no cluster writes, the renderer now preserves `RenderPathClearOnly`, keeps the published clear value, and leaves the packed-word stream empty instead of collapsing that frame to `Unavailable`.

The next N3 follow-up now also locks the first explicit `HardwareRasterizationPass` contract: the renderer-owned snapshot publishes execution-backed startup records for each rasterized cluster, and those records are sourced from a dedicated baseline-side pass seam rather than being rebuilt ad hoc from later host inspection helpers. Because `E:\Git\ZirconEngine\target` ran out of space during this continuation, the focused validation for this step was moved to `F:\cargo-targets\zircon-codex-b`.

The latest continuation extends that same seam through explicit provenance plus a real GPU buffer boundary: `RenderVirtualGeometryHardwareRasterizationSource` now keeps the renderer-owned/public-snapshot contract on `Unavailable`, `RenderPathClearOnly`, or `RenderPathExecutionSelections`, while the baseline pass itself constructs the startup buffer and returns `source + record_count + buffer` as one pass output. Even when there is no renderer-owned snapshot or uploader readback, the renderer still retains that hardware-rasterization startup parameter buffer and can decode it back into typed records, while the clear-only path remains observable as `RenderPathClearOnly` with an empty startup stream.

The latest continuation also closes the remaining public-stats gap on those two seams. `RenderStats` now mirrors both render-path sources plus both buffer/record counts, and `virtual_geometry_stats_contract` locks them against the renderer-owned snapshot when a real VG extract is present. On the opposite edge, a follow-up non-VG submission still resets those stats to `Unavailable` and `0`, so host tooling can distinguish “no effective VG workload” from “VG workload existed but the baseline pass only cleared state this frame.”

The latest follow-up closes the same public-stats gap for the executed `ClusterSelection` seam itself. `RenderStats` now also carries `last_virtual_geometry_selected_cluster_count`; `read_render_path_summary.rs` exposes the renderer-owned getter beside the existing `VisBuffer64` and hardware-rasterization summary accessors; and `update_stats/virtual_geometry_stats.rs` publishes that count only while an effective VG extract is present. That keeps public stats aligned with the execution-backed `selected_clusters` worklist rather than the broader visibility universe or the expanded indirect-draw count, and a later non-VG frame still clears the count back to `0`.

The next follow-up fixes the remaining provenance asymmetry on that same seam. `RenderVirtualGeometrySelectedClusterSource` now mirrors the pattern already established for `VisBuffer64` and hardware-rasterization startup records, with `Unavailable`, `RenderPathClearOnly`, and `RenderPathExecutionSelections` as the stable public states. The executed selected-cluster baseline pass now returns that source together with `selected_cluster_count + selected_cluster_buffer`; `store_last_runtime_outputs.rs` persists it onto both `RenderVirtualGeometryDebugSnapshot.selected_clusters_source` and the renderer-owned last-state; `read_render_path_summary.rs` and the direct renderer test helper `read_selected_cluster_source.rs` expose it without reopening a second inspection path; and `RenderStats` now mirrors the same source beside the selected-cluster count. That means the selected-cluster seam now exposes both quantity and provenance through the same public/debug surfaces as the other N3 baseline buffers.

The newest continuation removes the last duplicate reconstruction below that seam. `VirtualGeometryExecutedClusterSelectionPassOutput` now carries a typed `Vec<RenderVirtualGeometrySelectedCluster>` beside the selected-cluster source/count/buffer, `virtual_geometry_indirect_stats.rs`, `render.rs`, and `render_frame_with_pipeline.rs` thread that pass-owned list forward, and `store_last_runtime_outputs.rs` now routes snapshot persistence through `resolve_selected_clusters_for_store(...)`. When the render path already reported `RenderPathExecutionSelections` or `RenderPathClearOnly`, the store now persists the pass-owned typed list directly instead of rebuilding `snapshot.selected_clusters` a second time from `execution_segments + extract`; the older rebuild path remains only for `Unavailable`. This keeps the selected-cluster GPU buffer, provenance flag, public snapshot list, and downstream visbuffer/debug-mark rebuilds on one authoritative executed-cluster seam.

The next follow-up pushes `VisBuffer64` onto that same typed seam instead of keeping one more internal projection hop alive. `virtual_geometry_visbuffer64_pass.rs` now packs visibility words from `VirtualGeometryExecutedClusterSelectionPassOutput.selected_clusters` directly, using `RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(...)`, and `visbuffer64_pass_prefers_pass_owned_selected_clusters` now proves the baseline pass prefers that pass-owned typed list over the older internal `VirtualGeometryClusterSelection` projection whenever the executed selected-cluster seam is available. The internal selection DTO remains in place for the hardware-raster startup path, but `selected_cluster_buffer`, public `selected_clusters`, and render-path `VisBuffer64` packing now all consume the same cluster-identity source.

The newest continuation starts collapsing the same duplicate projection inside `virtual_geometry_hardware_rasterization_pass.rs` without widening ownership too early. Hardware-raster startup records are now built by zipping `VirtualGeometryExecutedClusterSelectionPassOutput.selections` with the pass-owned typed `selected_clusters`, sourcing identity and execution-state fields (`instance_index`, `entity`, `cluster_id`, `cluster_ordinal`, `page_id`, `lod_level`, `state`) from `RenderVirtualGeometrySelectedCluster` while keeping submission/startup-only metadata (`submission_index`, submission page/lod, cluster span/total count, lineage depth, frontier rank, slot ownership) on the internal `VirtualGeometryClusterSelection` seam. `hardware_rasterization_pass_prefers_pass_owned_selected_clusters` locks that split explicitly, so the remaining internal DTO surface is now constrained to startup parameters the public typed seam does not yet carry.

The newest continuation closes the matching store-time duplication on `VisBuffer64` itself. `VirtualGeometryVisBuffer64PassOutput` now carries typed `entries` beside `clear_value/source/entry_count/buffer`, `render.rs` and `render_frame_with_pipeline.rs` thread those entries forward, and `store_last_runtime_outputs.rs` now resolves snapshot/readback `visbuffer64_entries` through `resolve_visbuffer64_entries_for_store(...)`. When the render path already produced concrete `VisBuffer64` entries, the store now trusts that pass-owned logical stream directly instead of rebuilding a second entry list from `selected_clusters`; `RenderPathClearOnly` still keeps the public snapshot empty; and the uploader readback path retains an execution-backed rebuild fallback when prepare-only baseline frames have not yet materialized pass-owned entries. `resolve_visbuffer64_entries_for_store_prefers_pass_owned_entries`, `resolve_visbuffer64_entries_for_store_rebuilds_when_pass_entries_are_missing`, and `visbuffer64_pass_packs_words_from_pass_owned_entries` lock those seams explicitly.

The newest continuation extends that same execution-backed seam into `VirtualGeometryGpuReadback.selected_clusters` without over-claiming the not-yet-equivalent hardware-raster startup stream. `pending_readback/collect.rs` now seeds the CPU uploader DTO with an explicit `selected_clusters` field; `store_last_runtime_outputs.rs` backfills it from the already-resolved snapshot selection when that snapshot exists, and otherwise falls back to either the explicit render-path executed-cluster list or an `execution_segments + extract` rebuild when the selected-cluster pass only reported `RenderPathClearOnly` or stayed unavailable. `read_gpu_readback_selected_clusters.rs` then exposes that DTO through a non-consuming last-state helper, and `virtual_geometry_gpu_readback_exposes_execution_backed_visbuffer64_entries` now proves both the helper and the test-only `take_last_virtual_geometry_gpu_readback()` inspection path expose the same resident-cluster subset as the existing `VisBuffer64` readback contract. Production frame submission uses `take_last_virtual_geometry_gpu_completion_parts()` instead of taking the full readback owner, and the completion-part DTO itself is no longer crate-wide outside `graphics`.

The newest follow-up closes the matching provenance gap on that uploader `VisBuffer64` seam. `VirtualGeometryGpuReadback` now also carries `visbuffer64_source`; `pending_readback/collect.rs` seeds it as `Unavailable`; `store_last_runtime_outputs.rs` preserves the render-path `RenderVirtualGeometryVisBuffer64Source` on the readback DTO even when `visbuffer64_entries` were CPU-backfilled from execution-owned selected clusters; and `read_gpu_readback_visbuffer64_source.rs` exposes the source through a non-consuming helper. This still allows `visbuffer64_source == RenderPathClearOnly` on true zero-selection frames while `visbuffer64_entries` can hold an inspection-friendly fallback stream, but after the new render-time prepare synthesis the normal prepare-owned path now reports `RenderPathExecutionSelections` instead of collapsing into that mixed fallback state.

The newest continuation applies the same rule one seam later for raster startup provenance. `VirtualGeometryGpuReadback` now also carries `hardware_rasterization_source`; `pending_readback/collect.rs` seeds it as `Unavailable`; `store_last_runtime_outputs.rs` preserves the render-path `RenderVirtualGeometryHardwareRasterizationSource` on the uploader DTO even though the DTO still does not mirror the renderer-owned startup-record buffer; and `read_gpu_readback_hardware_rasterization_source.rs` exposes that source through a non-consuming helper. This keeps the DTO honest about whether the baseline `HardwareRasterizationPass` actually ran (`RenderPathExecutionSelections` or `RenderPathClearOnly`) without pretending the uploader readback owns a second copy of the startup parameter records.

The newest follow-up adds the matching scale signal without widening ownership. `VirtualGeometryGpuReadback` now also preserves `hardware_rasterization_record_count`; `pending_readback/collect.rs` seeds it as `0`; `store_last_runtime_outputs.rs` mirrors the render-path startup-record count onto the uploader DTO; and `read_gpu_readback_hardware_rasterization_record_count.rs` exposes that count through a non-consuming helper. The DTO still intentionally omits the startup-record payload itself, but host tooling can now distinguish `RenderPathClearOnly + 0`, `RenderPathExecutionSelections + N`, and `None` for no-uploader frames without reopening the renderer-owned startup buffer.

The newest continuation closes the same ambiguity for `selected_clusters` and `VisBuffer64` counts on the uploader seam. `read_gpu_readback_selected_cluster_count.rs` and `read_gpu_readback_visbuffer64_entry_count.rs` now expose the DTO-owned authoritative counts through non-consuming helpers, and `virtual_geometry_gpu.rs` now asserts those counts separately from the renderer-owned buffer counts. The important remaining mixed case is now the true zero-selection path, not the direct `with_virtual_geometry_prepare(...)` path: once `render.rs` synthesizes baseline `cluster_selections` from `virtual_geometry_prepare + extract.geometry.virtual_geometry` when explicit frame-owned selections are absent, prepare-owned frames publish `RenderPathExecutionSelections + 1` through the renderer-owned selected-cluster buffer, hardware-rasterization startup buffer, `VisBuffer64`, and the mirrored uploader source/count fields. CPU-side rebuilds remain only as the fallback for genuinely clear-only or unavailable frames.

The newest follow-up closes the direct-test gap between `VirtualGeometryPrepareFrame` and the baseline render path itself. `build_runtime_frame.rs` already populated `ViewportRenderFrame.virtual_geometry_cluster_selections` from prepare-owned truth for the normal runtime submission path, but direct renderer regressions that only called `.with_virtual_geometry_prepare(...)` still bypassed that seam and left `virtual_geometry_indirect_stats(...)` stuck on `RenderPathClearOnly`. `render.rs` now synthesizes the same selection list on demand when the frame carries `virtual_geometry_prepare` plus a real `RenderVirtualGeometryExtract` but no explicit `virtual_geometry_cluster_selections`, so both host-built runtime frames and direct renderer tests now feed the same executed-cluster baseline pass. `virtual_geometry_gpu_readback_exposes_execution_backed_visbuffer64_entries` locks the concrete teaching case: one resident cluster in prepare now yields renderer-owned `selected_clusters`, hardware-raster startup records, and `VisBuffer64` words together with uploader `source/count` mirrors at `RenderPathExecutionSelections + 1`, instead of the older `RenderPathClearOnly + CPU fallback list` split.

The newest continuation pulls that explicit-vs-prepare fallback up onto `ViewportRenderFrame` itself. The new helper `resolved_virtual_geometry_cluster_selections()` returns borrowed frame-owned selections when they already exist and otherwise materializes the prepare-derived worklist from `virtual_geometry_prepare + extract.geometry.virtual_geometry`. `render.rs` and `build_virtual_geometry_cluster_raster_draws.rs` now both consume that single resolver instead of open-coding the same fallback in two different places, and the new unit tests in `viewport_render_frame_resolve_virtual_geometry_cluster_selections.rs` lock the three intended states explicitly: borrowed explicit authority, owned prepare-derived authority, and `None` when the frame carries neither source.

The next follow-up fixes the mesh-build authority mismatch that the shared resolver exposed. `build_mesh_draw_build_context.rs` now treats frame-owned cluster selections as a true override only when they do not merely mirror `prepare.cluster_selections(extract)`. When the runtime frame is just carrying a mirrored prepare-derived worklist, mesh-build still respects `prepare.visible_entities` as the authoritative submit gate, which preserves the existing contract where prepare-owned visibility can trim the actual submitted mesh-draw subset even while the shared args buffers still keep the broader visibility-owned draw-ref universe. When the frame carries a genuine explicit override, mesh-build now follows that override instead. `build_context_prefers_explicit_cluster_selection_entities_over_prepare_visibility` locks the override case, while `build_context_keeps_prepare_visibility_when_frame_owned_selections_only_mirror_prepare_truth` captures the mirrored-prepare case so the runtime submission subset and the direct explicit-override path no longer collapse into the same behavior by accident.

The newest continuation pushes that same authority seam one layer higher into the future `NaniteGlobalStateBuffer` / `NodeAndClusterCull` boundary. `RenderVirtualGeometryDebugSnapshot` now carries a first-class `RenderVirtualGeometryCullInputSnapshot`, which freezes one host-visible DTO for the scalar inputs the later cull pass will consume: authored `cluster_budget/page_budget`, authored `instance/cluster/page` counts, current visible-entity gate, prepare-visible cluster count, current residency/request slot counts, the effective VG debug switches, and the current `cluster_selection_input_source`. `build_virtual_geometry_debug_snapshot.rs` assembles that DTO from `RenderVirtualGeometryExtract + VirtualGeometryPrepareFrame`, while `store_last_runtime_outputs.rs` patches the initial `Unavailable` source placeholder to the real frame-owned provenance (`ExplicitFrameOwned`, `PrepareDerivedFrameOwned`, or `PrepareOnDemand`) once the renderer knows which authority path actually fed the frame.

`RenderStats` now mirrors the lightweight authored-input scale on the public stats surface instead of forcing tools to open the full snapshot for basic budget accounting. The stats contract now exposes `last_virtual_geometry_cluster_budget`, `last_virtual_geometry_page_budget`, `last_virtual_geometry_input_cluster_count`, `last_virtual_geometry_input_page_count`, and `last_virtual_geometry_visible_entity_count` beside the already-existing debug/provenance fields, and a later non-VG frame still clears all five back to `0`. `debug_snapshot_builds_cull_input_snapshot_from_extract_and_prepare_state` locks DTO assembly at the submission-build layer, while `virtual_geometry_stats_contract` locks stats-vs-snapshot alignment on the public framework query path. That gives the current baseline path one stable host contract for “what would be fed into Nanite culling this frame” before any real GPU `NodeAndClusterCullPass` producer exists.

The newest continuation turns that DTO into a real renderer-owned GPU buffer instead of leaving it as snapshot-only metadata. `RenderVirtualGeometryCullInputSnapshot` now packs to and from a stable `u32` word layout, `render_frame_with_pipeline.rs` resolves a cull-input record even on direct renderer paths that never carried a framework-built debug snapshot, and `store_last_runtime_outputs.rs` now materializes `last_virtual_geometry_cull_input_buffer` from that record while preserving the final frame-owned `cluster_selection_input_source`. The new last-state helper `read_cull_input_snapshot.rs` decodes that buffer back into the typed DTO for tests and future inspection tooling. `virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback` locks the key seam: an explicit frame-owned `ClusterSelection` path with no framework snapshot and no uploader readback still produces a real cull-input buffer that round-trips the same authored budgets/debug/provenance surface. That is the first concrete `NaniteGlobalStateBuffer`-style buffer boundary in the current VG compatibility stack, even though the actual `NodeAndClusterCullPass` producer has not landed yet.

The newest continuation turns that raw buffer seam into the first explicit consumer-side `NodeAndClusterCull` startup bridge instead of leaving `last_virtual_geometry_cull_input_buffer` as orphaned metadata. `virtual_geometry_node_and_cluster_cull_pass.rs` now publishes `RenderVirtualGeometryNodeAndClusterCullSource::{Unavailable, RenderPathClearOnly, RenderPathCullInput}` plus `record_count + buffer`, and `render.rs`, `virtual_geometry_indirect_stats.rs`, `render_frame_with_pipeline.rs`, and `store_last_runtime_outputs.rs` thread that pass-owned output into `SceneRenderer` last-state. The important semantic choice is that this seam behaves like a future `NaniteGlobalStateBuffer`: whenever an effective VG extract exists, the pass still publishes one startup record even if downstream selected-cluster, hardware-rasterization, and `VisBuffer64` passes remain clear-only. The new helpers `read_node_and_cluster_cull_source.rs` and `read_node_and_cluster_cull_input_snapshot.rs` keep that pass-owned buffer testable, while `RenderStats` now mirrors `last_virtual_geometry_node_and_cluster_cull_source` plus `last_virtual_geometry_node_and_cluster_cull_record_count` so runtime preview and editor tooling can inspect the same seam without opening the full debug snapshot.

The newest follow-up closes the remaining public-query gap on that seam. `RenderVirtualGeometryDebugSnapshot` now also carries `node_and_cluster_cull_source` plus `node_and_cluster_cull_record_count`, `build_virtual_geometry_debug_snapshot.rs` seeds them as `Unavailable / 0` at submission-build time, and `store_last_runtime_outputs.rs` backfills the real render-path values from the pass-owned `NodeAndClusterCull` output just before persisting the renderer-owned snapshot. That keeps all three host surfaces aligned on the same contract: renderer-private last-state for low-level readback, `RenderStats` for lightweight counters/provenance, and `query_virtual_geometry_debug_snapshot()` for one full public inspection object.

The newest follow-up widens that same startup seam from a raw cull-input payload into a typed global-state record. `RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot` now packs `cull_input + viewport_size + camera_translation + view_proj` into the startup buffer layout, `virtual_geometry_node_and_cluster_cull_pass.rs` keeps the typed record beside the GPU buffer it uploads, and the new helper `read_node_and_cluster_cull_global_state_snapshot.rs` plus the direct renderer regression in `virtual_geometry_gpu.rs` lock that the packed buffer round-trips the same viewport/camera/view-projection inputs the renderer already uses for scene-uniform setup.

The newest continuation then lifts that typed record onto the public framework query surface instead of stopping at renderer-private readback helpers. `RenderVirtualGeometryDebugSnapshot` now also carries `node_and_cluster_cull_global_state`, `render.rs` and `render_frame_with_pipeline.rs` thread the typed pass output through without adding a second GPU readback, and `store_last_runtime_outputs.rs` backfills the final frame-owned `cluster_selection_input_source` onto both `snapshot.cull_input` and `snapshot.node_and_cluster_cull_global_state.cull_input` so the public snapshot exposes one internally consistent `NaniteGlobalStateBuffer`-style view. `virtual_geometry_debug_snapshot_contract.rs` now locks that the framework query path exposes the same final cull-input DTO together with the correct viewport size, camera origin, and view-projection matrix.

The newest follow-up finally gives that root-seed worklist its first downstream render-path consumer instead of leaving it as inspection-only startup data. `virtual_geometry_indirect_stats.rs` now executes `NodeAndClusterCull` before `virtual_geometry_executed_cluster_selection_pass.rs`, and the executed-cluster baseline pass still prefers explicit or prepare-derived `ClusterSelection` input but can now synthesize bounded execution candidates from `RenderVirtualGeometryNodeAndClusterCullInstanceSeed` when no `ClusterSelection` exists. The first landed policy took one cluster per seeded instance range; the next step widens that same seam to expand every cluster in the seeded range while still respecting the shared `cluster_budget` clamp from `RenderVirtualGeometryCullInputSnapshot`; and the next follow-up threads `forced_mip` from the same typed cull/global-state record into that range expansion so manual mip forcing prunes the baseline execution worklist before it reaches selected-cluster, hardware-rasterization, or `VisBuffer64`. Execution state still comes directly from page residency (`Resident`, `PendingUpload`, or `Missing`), and the resulting worklist still feeds the existing execution-owned selected-cluster, hardware-rasterization, and `VisBuffer64` seams without widening any public source enum. This is still not real BVH traversal yet; it is the baseline range-expansion bridge that lets downstream passes start consuming `NodeAndClusterCull` output through the same multi-cluster buffer contracts that real `VisitNode / StoreCluster` GPU logic will keep.

The newest follow-up closes the remaining hardcoded startup-metadata gap on that same seed-backed seam. `virtual_geometry_executed_cluster_selection_pass.rs` now builds a `clusters_by_id` lookup from the active extract and derives `lineage_depth` for every seed-expanded cluster by walking its `parent_cluster_id` chain with the same cycle-guarded semantics already used by the visibility planner, instead of hardcoding every seed-expanded cluster to depth `0`. This keeps the current `NodeAndClusterCull -> executed-cluster -> hardware-rasterization` baseline bridge aligned with the existing BVH teaching model: parent clusters stay at depth `0`, child clusters carry depth `1`, and deeper descendants keep incrementing.

The newest follow-up closes the adjacent subset-range metadata gap on that same baseline seam. When a `NodeAndClusterCull` instance seed references only a subset of an instance's cluster slice, the old baseline path reused the raw extract slice offset as both `cluster_ordinal` and `entity_cluster_start_ordinal`; that made the selected-cluster and hardware-rasterization seams report an unstable ordinal even though the active seed range itself only contained one cluster. `virtual_geometry_executed_cluster_selection_pass.rs` now builds a seed-backed ordering map from the effective instance slices, sorts those slices into the same stable per-instance `cluster_id` order already expected by the rest of the VG host path, and uses that ordering to populate `cluster_ordinal` plus `entity_cluster_start_ordinal` for seed-expanded selections. The important non-change is that `entity_cluster_total_count` still follows the current seeded instance slice rather than inventing a broader per-entity count, so this fix only corrects the bad ordinal/start metadata instead of silently widening the draw segment contract.

The newest follow-up closes the adjacent resident-parent fallback gap on that same baseline seam. When `forced_mip` is not active and a seed-expanded cluster resolves to `PendingUpload` or `Missing`, `virtual_geometry_executed_cluster_selection_pass.rs` now walks `parent_cluster_id` upward with a cycle guard and substitutes the nearest resident ancestor cluster before publishing the execution-owned selected-cluster, hardware-rasterization, and `VisBuffer64` worklists. Deduplication now happens on the resolved cluster id, which means an undrawable child no longer survives as a separate work item if it collapses onto an ancestor that was already selected earlier in the same seeded instance slice. This is the current baseline version of Nanite-style parent residency fallback: it improves drawability without inventing a second public worklist shape, while `forced_mip` intentionally disables the replacement so exact-mip teaching/debug inspection remains authoritative.

The newest follow-up closes the remaining hardcoded frontier-order gap on that same baseline seam. `virtual_geometry_executed_cluster_selection_pass.rs` now carries a seed-backed unresolved-page ranking state and assigns `frontier_rank` from the first occurrence of each nonresident page in the expanded execution worklist instead of reporting every seed-expanded cluster as rank `0`. Resident clusters still default to `0`, while `PendingUpload` and `Missing` pages receive stable per-page ranks in encounter order; that keeps the seam aligned with the existing prepare/runtime notion that frontier ordering belongs to page pressure, not to cluster id ordering. This is still a baseline proxy rather than true traversal-owned frontier ranking, but it gives hardware-rasterization and later debug/output seams a stable unresolved-page order before real `VisitNode / StoreCluster` GPU traversal owns that field outright.

Focused validation for that seed-consumer step stayed green on `F:\cargo-targets\zircon-codex-b` with the renderer regressions `seed_backed_node_and_cluster_cull_can_drive_execution_selected_clusters_without_explicit_cluster_selections`, `seed_backed_node_and_cluster_cull_can_drive_multiple_execution_selected_clusters_without_explicit_cluster_selections`, `seed_backed_node_and_cluster_cull_respects_forced_mip_without_explicit_cluster_selections`, `seed_backed_node_and_cluster_cull_preserves_lineage_depth_in_hardware_rasterization_records_without_explicit_cluster_selections`, `seed_backed_node_and_cluster_cull_keeps_instance_local_cluster_slice_metadata_for_subset_seed_ranges_without_explicit_cluster_selections`, `seed_backed_node_and_cluster_cull_falls_back_to_resident_parent_cluster_without_explicit_cluster_selections`, and `seed_backed_node_and_cluster_cull_derives_frontier_rank_from_unresolved_page_order_without_explicit_cluster_selections`; the pass-local helper regressions `seed_backed_execution_selection_expands_all_clusters_in_seed_range_and_page_residency`, `seed_backed_execution_selection_respects_forced_mip`, `seed_backed_execution_selection_derives_lineage_depth_from_parent_chain`, `seed_backed_execution_selection_keeps_instance_local_cluster_ordinal_for_subset_seed_ranges`, `seed_backed_execution_selection_falls_back_to_nearest_resident_parent_cluster`, and `seed_backed_execution_selection_derives_frontier_rank_from_first_unresolved_page_occurrence`; the existing empty-extract guard `virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections`; the existing executed-cluster ordering regression `executed_cluster_selection_pass_filters_deduplicates_and_sorts_cluster_selections`; and a final `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`. The renderer regressions live in `virtual_geometry_node_and_cluster_cull_execution.rs` specifically to keep the overgrown `virtual_geometry_gpu.rs` test bucket from absorbing yet another unrelated responsibility while still locking the single-cluster budget-clamped path, the multi-cluster range-expansion path, the manual `forced_mip` filter, the lineage-depth startup metadata, the corrected subset-range ordinal/start metadata, the resident-parent fallback semantics, and the unresolved-page `frontier_rank` proxy on that same baseline seam. The lineage-depth renderer fixture now keeps ancestor pages nonresident on purpose so that test continues to isolate parent-chain metadata propagation rather than being silently satisfied by fallback substitution.

The newest continuation closes the remaining ownership gap between `NodeAndClusterCull.instance_seeds` and the seed-backed executed-cluster baseline seam. `virtual_geometry_executed_cluster_selection_pass.rs` is now split so the seed-backed ordering, fallback, and frontier helpers live under `virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection.rs` instead of continuing to accumulate inside the pass entry file, and the root-seed baseline path now builds submission ordering from the authoritative `instance_seeds` ranges rather than silently drifting back to the broader `extract.instances` slice. The new pass-local regression `seed_backed_execution_selection_collection_uses_node_and_cluster_cull_seed_range_as_the_authoritative_submission_slice` locks the key future-facing case: when a later GPU cull pass narrows one instance into a smaller seed range, `cluster_ordinal`, `entity_cluster_start_ordinal`, `entity_cluster_total_count`, and the published selected-cluster ordinal now reset to that seed-local worklist instead of reusing the broader extract-local order. Focused validation for this slice stayed green on `F:\cargo-targets\zircon-codex-b` with `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_collection_uses_node_and_cluster_cull_seed_range_as_the_authoritative_submission_slice --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_node_and_cluster_cull_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b executed_cluster_selection_pass_ --lib -- --nocapture`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`. The remaining structure debt is now explicit instead of hidden: the pass entry file is smaller because the production seed-backed logic moved out, but its inline regression block is still oversized, so the next non-behavioral extraction boundary should be a dedicated `virtual_geometry_executed_cluster_selection_pass/tests/` subtree rather than piling more pass-local tests back into the root module.

That structural follow-up is now complete. The pass root keeps only the entry wiring plus `#[cfg(test)] mod tests;`, the seed-backed behavior remains isolated in `virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection.rs`, and the pass-local regression mass now lives under `virtual_geometry_executed_cluster_selection_pass/tests/{selection_filter,seed_backed_ranges,seed_backed_fallbacks,seed_backed_ordering}.rs` with shared fixtures in `tests/support.rs`. After this extraction the root pass is back to a small module instead of a mixed production-and-regression body, which clears the last local structure debt on the current `NodeAndClusterCull -> executed-cluster` baseline seam before the first explicit `NaniteGlobalStateBuffer` dispatch/setup continuation. Focused validation for the structural split stayed green on `F:\cargo-targets\zircon-codex-b` with `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b executed_cluster_selection_pass_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_node_and_cluster_cull_ --lib -- --nocapture`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`.

The newest continuation consumes the typed `node_and_cluster_cull_global_state` record into the first explicit `NodeAndClusterCull` dispatch/setup seam instead of leaving dispatch math implicit in host code. `RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot` now packs `(instance_seed_count, cluster_budget, page_budget, workgroup_size, dispatch_group_count)` into a stable GPU word layout; `virtual_geometry_node_and_cluster_cull_pass.rs` derives that record directly from `global_state + instance_seeds`; `render.rs`, `render_frame_with_pipeline.rs`, and `store_last_runtime_outputs.rs` preserve the dedicated dispatch-setup buffer on `SceneRenderer`; `read_node_and_cluster_cull_dispatch_setup_snapshot.rs` exposes the renderer-owned readback; and `RenderVirtualGeometryDebugSnapshot` now mirrors the same typed record so framework queries, runtime preview, and editor inspection see the exact startup work plan. This is still a setup seam rather than real BVH compute traversal, but the future `NaniteGlobalStateBuffer + dispatch` path is now an explicit contract instead of an inferred host-side convention. Focused validation for this slice stayed green on `F:\cargo-targets\zircon-codex-b` with `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b render_framework_exposes_virtual_geometry_debug_snapshot_for_effective_visible_clusters --test virtual_geometry_debug_snapshot_contract -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_node_and_cluster_cull_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b executed_cluster_selection_pass_ --lib -- --nocapture`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`.

Focused validation for this convergence stayed green on `F:\cargo-targets\zircon-codex-b` with `cull_input_snapshot_roundtrips_through_gpu_word_layout`, `node_and_cluster_cull_`, the direct renderer regression `virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback`, `virtual_geometry_debug_snapshot_contract`, and a final `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`. The integration-test rerun also exposed an unrelated Hybrid GI scene-truth revision-chain compile drift: `runtime_parent_chain.rs` now owns dedicated `support + revision` lineage helpers, and `hybrid_gi_temporal_signature.rs` consumes those instead of overloading the older `support + quality` helpers. That repair is support-only, but it keeps the shared `zircon_runtime` compile/test path open for the VG contract work above.

The newest continuation closes the matching debug-override gap on the automatic cooked-VG path. `build_frame_submission_context/build.rs` now passes `extract.geometry.virtual_geometry_debug` into `SceneRenderer::synthesize_virtual_geometry_extract(...)`, `scene_renderer_virtual_geometry/automatic_extract.rs` forwards that override through a dedicated `build_virtual_geometry_automatic_extract_from_meshes_with_debug(...)` entry point, and `nanite/automatic_extract.rs` now maps the public render debug state into `VirtualGeometryCpuReferenceConfig` before constructing `VirtualGeometryCpuReferenceFrame`. This matters because the automatic CPU-reference/BVH inspection surfaces are supposed to be the N2 teaching/debug truth for cooked assets; before this fix, they always used default config and could silently disagree with the effective extract whenever `forced_mip` was set. `render_framework_automatic_virtual_geometry_bvh_selected_clusters_follow_forced_mip_override` now locks the intended contract: a resident cluster that fails `forced_mip` stays resident in BVH visualization but drops out of the selected-cluster set.

The newest follow-up closes the remaining public-state mismatch on that same path. `nanite/automatic_extract.rs` now converts `VirtualGeometryCpuReferenceConfig` back into `RenderVirtualGeometryDebugState` when it builds the returned `VirtualGeometryAutomaticExtractOutput`, so `output.extract.debug` no longer drifts back to defaults while `cpu_reference_instances` and `bvh_visualization_instances` already reflect the requested override. The new unit test `virtual_geometry_nanite_mesh_based_automatic_extract_with_debug_keeps_extract_debug_in_sync` locks that contract for the mesh-snapshot/model-resolver synthesis seam.

The newest follow-up extends the same N2 teaching surface with explicit per-layer cluster output. `RenderVirtualGeometryCpuReferenceInstance` now carries `depth_cluster_map` entries, backed by the new `RenderVirtualGeometryCpuReferenceDepthClusterMapEntry` DTO, and `nanite/automatic_extract.rs` derives that list directly from the CPU-reference node visits so hosts can dump the cluster set for each BVH depth without re-deriving layer groupings themselves. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_clusters_grouped_by_bvh_depth` and the updated framework snapshot contract lock that per-layer view on both the helper and renderer-owned inspection paths.

The newest follow-up turns the plan's `VisitNode / StoreCluster` terminology into concrete CPU-reference code structure. `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/cpu_reference.rs` now owns an internal `VirtualGeometryCpuReferenceTraversalState` plus explicit `visit_node(...)` and `store_cluster(...)` helpers, and `VirtualGeometryCpuReferenceFrame::from_asset(...)` now builds its visited-node, leaf-cluster, and selected-cluster outputs through those helpers instead of one monolithic inline traversal loop. The new unit tests `visit_node_records_visit_order_and_cluster_ids` and `store_cluster_keeps_all_leafs_and_selects_only_resident_matching_mip` lock those semantics directly before later GPU hierarchical culling grows around the same vocabulary.

The newest follow-up adds the matching mip-level teaching surface. `RenderVirtualGeometryCpuReferenceInstance` now also carries `mip_cluster_map`, backed by `RenderVirtualGeometryCpuReferenceMipClusterMapEntry`, and `nanite/automatic_extract.rs` derives it from the CPU-reference leaf-cluster list so hosts can print the full leaf mip distribution and isolate a concrete mip such as `10` without replaying the selection logic. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_leaf_clusters_grouped_by_mip` and the updated renderer-owned snapshot contract lock that view on both helper and framework query paths.

The newest follow-up finishes the earlier `cluster_ordinal` teaching-surface gap instead of leaving ordinals implicit inside the traversal loop. `VirtualGeometryCpuReferenceLeafCluster` and `RenderVirtualGeometryCpuReferenceLeafCluster` now both preserve `cluster_ordinal`, and the traversal regression `virtual_geometry_nanite_cpu_reference_traverses_hierarchy_maps_pages_and_filters_forced_mip` now asserts the concrete ordering `(0,100)`, `(1,200)`, `(2,300)` on the leaf side plus `(0,100)`, `(2,300)` on the selected side. This matters because the CPU reference is the golden N2 teaching path for the later `ClusterSelection` / `NodeAndClusterCull` worklist; once the leaf ordinal is explicit, host tools and later GPU-facing seams can talk about stable cluster order without reconstructing it from hierarchy-local offsets.

The newest continuation then exposes that same selected subset directly on the public CPU-reference inspection DTO instead of forcing hosts to recompute it from `leaf_clusters + residency + forced_mip`. `RenderVirtualGeometryCpuReferenceInstance` now carries `selected_clusters`, backed by the new `RenderVirtualGeometryCpuReferenceSelectedCluster` DTO, and `nanite/automatic_extract.rs` maps it straight from `VirtualGeometryCpuReferenceFrame.selected_clusters` while keeping the existing full-detail `leaf_clusters` list for geometry/debug metadata. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_selected_clusters_as_worklist` plus the updated `virtual_geometry_debug_snapshot_contract.rs` lock the intended contract: the helper surface and the renderer-owned snapshot now both expose the exact post-selection worklist with stable `cluster_ordinal`, `page_id`, `mip_level`, and residency truth, which is the CPU mirror the later GPU `ClusterSelection` pass will replace rather than reinterpret.

The newest follow-up closes the adjacent N2 residency-inspection gap by exposing loaded leafs separately from selected leafs. `RenderVirtualGeometryCpuReferenceInstance` now also carries `loaded_leaf_clusters`, reusing the full `RenderVirtualGeometryCpuReferenceLeafCluster` payload but filtering it down to `loaded == true` inside `nanite/automatic_extract.rs`. This is intentionally not the same as `selected_clusters`: the loaded-leaf list answers “which BVH leaf clusters already have resident page data,” while the selected worklist still answers “which loaded leaf clusters survived the current `forced_mip` / selection rules.” The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_loaded_leaf_clusters_as_worklist` locks that distinction explicitly with `forced_mip=9`, where the loaded leafs remain `(0,100)` and `(2,300)` but the selected worklist becomes empty; the updated renderer-owned snapshot contract keeps the same explicit loaded-leaf view on the framework query path. This directly covers the plan's “已加载分页后的叶子验证” teaching requirement without forcing host tools to replay the residency filter over the full leaf list.

The newest follow-up exposes the missing middle step between the full leaf list and the final selected worklist: the current mip-selector result before residency gating. `RenderVirtualGeometryCpuReferenceInstance` now also carries `mip_accepted_clusters`, reusing the full `RenderVirtualGeometryCpuReferenceLeafCluster` payload, and `nanite/automatic_extract.rs` derives it from `leaf_clusters` plus the effective `forced_mip` in `VirtualGeometryCpuReferenceFrame.debug`. This surface answers “which leaf clusters survive the current manual mip selector,” regardless of whether their pages are resident yet. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_mip_accepted_clusters_as_worklist` locks the important teaching case with `forced_mip=9`: `mip_accepted_clusters` reports `(1,200)` even though `selected_clusters` stays empty because page `20` is still not resident. The updated renderer-owned snapshot contract keeps that same worklist on the framework query path, so host tools can now explain “mip matched but page missing” without re-deriving the distinction from `mip_cluster_map + loaded_leaf_clusters + selected_clusters` themselves.

The newest follow-up carries that same pre-residency mip-selector surface into page space. `RenderVirtualGeometryCpuReferenceInstance` now also carries `mip_accepted_page_cluster_map`, reusing `RenderVirtualGeometryCpuReferencePageClusterMapEntry`, and `nanite/automatic_extract.rs` derives it from the `forced_mip`-accepted subset of `leaf_clusters` before residency gating. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_mip_accepted_page_cluster_map` locks the teaching case directly with `forced_mip=9`: even while `selected_clusters` remains empty, the mip-accepted page map reports `20 -> [200]`. The updated renderer-owned snapshot contract keeps that grouped page answer on the framework query path, so host tools can now explain not just “which cluster matched the manual mip selector,” but also “which page that selector is currently asking for.”

The newest follow-up pushes that same residency teaching surface one step closer to the plan's page-debug tasks by exposing the loaded page-to-cluster map directly. `RenderVirtualGeometryCpuReferenceInstance` now carries `loaded_page_cluster_map`, reusing `RenderVirtualGeometryCpuReferencePageClusterMapEntry`, and `nanite/automatic_extract.rs` derives it from the loaded subset of `leaf_clusters` instead of making host tools regroup `loaded_leaf_clusters` themselves. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_loaded_page_cluster_map` locks the same `forced_mip=9` distinction: even while the selected worklist stays empty, the loaded page map still reports page `10 -> [100]` and page `30 -> [300]`. The updated renderer-owned snapshot contract keeps that page-residency view aligned across helper and framework query paths, which gives the N2 CPU reference a direct answer to “哪些 page 已经就绪，以及每个已加载 page 上挂了哪些叶子 cluster” before later GPU residency and cull passes replace the producer.

The newest follow-up extends that same residency-only view into the mip dimension. `RenderVirtualGeometryCpuReferenceInstance` now also carries `loaded_mip_cluster_map`, again reusing `RenderVirtualGeometryCpuReferenceMipClusterMapEntry`, and `nanite/automatic_extract.rs` derives it from the `loaded == true` subset of `leaf_clusters` instead of forcing host tools to regroup `loaded_leaf_clusters` by mip. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_loaded_mip_cluster_map` locks the important `forced_mip=9` distinction: the selected worklist still stays empty, but the loaded mip map continues to report `10 -> [100, 300]` because residency and selection are intentionally different questions. The updated renderer-owned snapshot contract keeps that loaded-mip grouping aligned across helper and framework query paths, which gives the N2 CPU teaching surface a direct answer to “当前已经常驻的叶子 cluster 分布在什么 mip 桶里” before automatic GPU culling starts producing mixed selected mip sets.

The newest follow-up also adds the residency-only depth view so the loaded subset can be compared against the full BVH traversal without host-side regrouping. `RenderVirtualGeometryCpuReferenceInstance` now carries `loaded_depth_cluster_map`, still reusing `RenderVirtualGeometryCpuReferenceDepthClusterMapEntry`, and `nanite/automatic_extract.rs` derives it by joining the loaded subset of `leaf_clusters` back against `visited_nodes` to recover BVH depth. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_loaded_depth_cluster_map` locks that view at `1 -> [100, 300]`, while the updated renderer-owned snapshot contract keeps the same grouping on the framework query path. At this point the N2 CPU inspect surface has parallel all-vs-loaded-vs-selected answers across the main teaching dimensions: depth, mip, and page. That is the right CPU-side shape before later `NodeAndClusterCull` work starts producing its own execution-backed loaded/selected distinctions on GPU.

The newest follow-up extends that same depth-regrouping surface to the pre-residency mip-selector result. `RenderVirtualGeometryCpuReferenceInstance` now also carries `mip_accepted_depth_cluster_map`, again reusing `RenderVirtualGeometryCpuReferenceDepthClusterMapEntry`, and `nanite/automatic_extract.rs` derives it by joining the `forced_mip`-accepted subset of `leaf_clusters` back against `visited_nodes` before residency gating. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_mip_accepted_depth_cluster_map` locks the key teaching case with `forced_mip=9`: even while `selected_clusters` stays empty, the mip-accepted depth map reports `1 -> [200]`. The updated renderer-owned snapshot contract keeps that grouped depth answer on the framework query path, so host tools can now explain not just “the mip selector chose page 20,” but also “that choice currently lives at BVH depth 1.”

The newest follow-up closes the matching selection-side regrouping gap so the page views are now explicit instead of implicit. `RenderVirtualGeometryCpuReferenceInstance` now also carries `selected_page_cluster_map`, still reusing `RenderVirtualGeometryCpuReferencePageClusterMapEntry`, and `nanite/automatic_extract.rs` derives it from `VirtualGeometryCpuReferenceFrame.selected_clusters`. That gives the CPU-reference inspect surface four page answers with different semantics: `page_cluster_map` for all cooked leaf membership, `loaded_page_cluster_map` for residency, `mip_accepted_page_cluster_map` for the current pre-residency mip selector, and `selected_page_cluster_map` for the current post-selection worklist. The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_selected_page_cluster_map` locks the selected-page view directly at `10 -> [100]` and `30 -> [300]`, and the updated renderer-owned snapshot contract keeps that same grouping available on the framework query path. This makes the N2 CPU debug surface line up more closely with the later `ClusterSelection -> page decode` GPU handoff, because host tools no longer need to regroup selected clusters before comparing CPU and GPU page-local worklists.

The newest follow-up does the same thing for the mip dimension so current LOD choice becomes explicit instead of inferred. `RenderVirtualGeometryCpuReferenceInstance` now also carries `selected_mip_cluster_map`, reusing `RenderVirtualGeometryCpuReferenceMipClusterMapEntry`, and `nanite/automatic_extract.rs` derives it from `VirtualGeometryCpuReferenceFrame.selected_clusters`. This complements the existing full-leaf `mip_cluster_map`: the full map still answers “which cooked leafs exist at each mip,” while the selected map answers “which mip buckets actually survived the current selection rules this frame.” The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_selected_mip_cluster_map` locks the intended distinction with `forced_mip=10`, where the full mip map still reports both `9 -> [200]` and `10 -> [100, 300]` but the selected mip map collapses to `10 -> [100, 300]`. The updated renderer-owned snapshot contract keeps that selected-mip grouping on the framework query path, which gives the N2 CPU teaching surface a direct per-frame LOD answer before the later automatic GPU cull path starts choosing mixed mip sets across the scene.

The newest follow-up applies the same idea to BVH depth so the selected worklist can be compared against the full traversal tree without host-side regrouping. `RenderVirtualGeometryCpuReferenceInstance` now also carries `selected_depth_cluster_map`, reusing `RenderVirtualGeometryCpuReferenceDepthClusterMapEntry`, and `nanite/automatic_extract.rs` derives it by joining `selected_clusters` back against `visited_nodes` to recover each selected cluster's BVH depth. This complements the existing full `depth_cluster_map`: the full map still answers “which cluster ids were encountered at each visited depth,” while the selected map answers “which of those depth buckets actually survived the current selection rules.” The new unit test `virtual_geometry_nanite_cpu_reference_instances_expose_selected_depth_cluster_map` locks the selected-depth view at `1 -> [100, 300]`, and the updated renderer-owned snapshot contract keeps that same grouping on the framework query path. At this point the N2 CPU inspect surface has explicit full, loaded, mip-accepted, and selected views where that distinction adds explanatory value across the main teaching dimensions, which is the right CPU-side shape before GPU culling starts producing its own selected-depth worklists.

As supporting validation work, this continuation also needed a support-only Hybrid GI readback compile repair in `gpu_readback/pending_readback/collect.rs` so the focused framework-level VG snapshot query could reach the intended red/green loop again. The same validation loop then needed one more support-only Hybrid GI compile repair after `HybridGiScenePrepareFrame` and `HybridGiPrepareExecutionInputs` widened their scene surface-cache page-content fields: legacy Hybrid GI GPU/resolve fixtures plus `voxel_clipmap_debug.rs` now seed those fields with empty vectors so `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b resolve_selected_clusters_for_store_prefers_pass_owned_selected_clusters --lib -- --nocapture` can compile and run again. These support fixes do not alter VG ownership boundaries or expand this slice into Hybrid GI feature work.

The newest continuation pushes that same startup chain one step lower and makes `instance_work_items` the first real compute-stub output below `launch_worklist`. `RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem` now owns a stable GPU layout for `(instance_index, entity, cluster/page slice, cluster_budget, page_budget, forced_mip)`; `virtual_geometry_node_and_cluster_cull_pass.rs` mirrors the DTO on CPU, dispatches `node_and_cluster_cull_instance_work_items.wgsl` through dedicated GPU resources, and publishes `instance_work_item_count + instance_work_item_buffer` beside the older startup buffers; `render.rs`, `render_frame_with_pipeline.rs`, `scene_renderer.rs`, `new_with_icon_source.rs`, `reset_last_runtime_outputs.rs`, and `store_last_runtime_outputs.rs` preserve that seam on renderer-owned last-state; `read_node_and_cluster_cull_instance_work_items.rs` exposes typed GPU readback; and `RenderVirtualGeometryDebugSnapshot.node_and_cluster_cull_instance_work_items` now mirrors the same worklist on the public framework query surface. The baseline seam also moved accordingly: `seed_backed_execution_selection.rs` now treats `instance_work_items` as the authoritative contract below `launch_worklist`, so the guard regression is `seed_backed_execution_selection_collection_requires_explicit_instance_work_item_contract` rather than the older seed-only launch guard. Focused validation for this slice stayed green on `F:\cargo-targets\zircon-codex-b` with `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_instance_work_item_roundtrips_through_gpu_word_layout --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_publishes_instance_work_items_from_launch_worklist_contract --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_collection_requires_explicit_instance_work_item_contract --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_debug_snapshot_contract -- --nocapture`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`.

The newest continuation pushes that same baseline chain one level lower without widening the public framework DTO surface again. `virtual_geometry_node_and_cluster_cull_pass.rs` now derives an internal `cluster_work_items` seam from `instance_work_items`, expanding each per-instance slice into one typed per-cluster row `(instance_index, entity, cluster_array_index, cluster_budget, page_budget, forced_mip)`. `seed_backed_execution_selection.rs` now requires both `instance_work_items` and `cluster_work_items`, builds seed-backed ordering from those explicit cluster rows, and emits execution candidates directly from `cluster_array_index` instead of re-scanning broad instance ranges from the extract. That design is deliberate: `instance_work_items` remains the public renderer/debug contract below `launch_worklist`, while `cluster_work_items` is the first tighter internal bridge toward `VisitNode / StoreCluster` and later GPU cull kernels. The new pass regression `node_and_cluster_cull_pass_publishes_cluster_work_items_from_instance_work_item_contract`, the new baseline guard `seed_backed_execution_selection_collection_requires_explicit_cluster_work_item_contract`, and the retained `seed_backed_execution_selection_collection_requires_explicit_instance_work_item_contract` now lock that the baseline path refuses to synthesize cluster candidates once either seam disappears. This slice also required another target cleanup before validation because `F:` had fallen to `0.70 GB` free; after `cargo clean --target-dir F:\cargo-targets\zircon-codex-b` reclaimed `7.0 GiB`, the focused rerun set stayed green with `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_publishes_cluster_work_items_from_instance_work_item_contract --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_collection_requires_explicit_cluster_work_item_contract --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_collection_requires_explicit_instance_work_item_contract --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_node_and_cluster_cull_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b executed_cluster_selection_pass_ --lib -- --nocapture`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`. The remaining `cargo check` warnings are currently confined to the unrelated Hybrid GI files already noted elsewhere in this document.

The newest follow-up materializes that `cluster_work_items` seam as renderer-owned GPU state instead of leaving it as pass-local CPU data. `VirtualGeometryNodeAndClusterCullClusterWorkItem` now lives in `graphics::types` with a stable eight-word layout for `(instance_index, entity, cluster_array_index, cluster_budget, page_budget, forced_mip)`, `virtual_geometry_node_and_cluster_cull_pass.rs` publishes `cluster_work_item_count + cluster_work_item_buffer`, and `render.rs`, `render_frame_with_pipeline.rs`, `scene_renderer.rs`, `new_with_icon_source.rs`, `reset_last_runtime_outputs.rs`, and `store_last_runtime_outputs.rs` preserve that buffer through last-state. The test helper `read_node_and_cluster_cull_cluster_work_items.rs` decodes the GPU buffer back into typed queue rows, and `virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback` now proves the renderer can recover the two per-cluster work rows directly from GPU last-state after a real frame. Focused validation for the slice included the cluster work-item word-layout regression, the renderer-owned readback contract, the pass-level `node_and_cluster_cull_pass_` suite, and the seed-backed baseline guard that requires explicit cluster work items. A later hierarchy-identity follow-up widened this work-item layout to nine words by adding optional `hierarchy_node_id`, encoded with the same `u32::MAX` sentinel convention as optional forced mip.

The latest continuation stops treating `cluster_work_items` as the terminal cull seam by adding the first typed traversal-side consumer below it. `VirtualGeometryNodeAndClusterCullTraversalRecord` now publishes explicit `VisitNode`, `StoreCluster`, and `EnqueueChild` queue rows with a stable thirteen-word layout that preserves operation, child-range provenance, instance ownership, entity id, cluster row, child base/count, traversal index, budgets, and optional forced mip. `virtual_geometry_node_and_cluster_cull_pass.rs` first emitted a deterministic `VisitNode -> StoreCluster` pair for each cluster work item, then the adjacent budget follow-up made that producer decision-bearing: every candidate still receives a `VisitNode` audit row, only the first `cluster_budget` candidates receive `StoreCluster`, and over-budget candidates now receive `EnqueueChild` instead of disappearing into a visited-only gap. The fanout follow-up gives that child-work marker explicit `child_base + child_count` payload using deterministic four-way child ranges derived from the current cluster row; the provenance follow-up marks that payload as `FixedFanout` while `VisitNode` and `StoreCluster` records carry `None`. The pass publishes both CPU records and `traversal_record_count + traversal_record_buffer`; `render.rs`, `render_frame_with_pipeline.rs`, `scene_renderer.rs`, `new_with_icon_source.rs`, `reset_last_runtime_outputs.rs`, and `store_last_runtime_outputs.rs` preserve that buffer through renderer-owned last-state. The test helper `read_node_and_cluster_cull_traversal_records.rs` decodes the GPU buffer back into typed rows, and `virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback` verifies the two-cluster frame exposes the ordered traversal records directly from GPU last-state. This is intentionally still not real BVH/SSE splitting; it pins the queue contract, the first budget-gated keep/drop distinction, and an explicit child-work payload with provenance so a later GPU hierarchy kernel can replace the baseline producer without reopening the older broad extract scans.

The newest hierarchy-identity follow-up gives that traversal seam the asset-authored node id it was missing. `RenderVirtualGeometryCluster` now carries optional `hierarchy_node_id`; the Zircon-native CPU reference path forwards `VirtualGeometryCpuReferenceLeafCluster.node_id`, and cooked automatic extraction forwards `VirtualGeometryClusterHeaderAsset.hierarchy_node_id`. `virtual_geometry_node_and_cluster_cull_pass.rs` looks up each cluster work item against the effective render extract cluster row, then carries the same id into `VirtualGeometryNodeAndClusterCullTraversalRecord`; the traversal record GPU layout therefore widens from thirteen to fourteen words. Current `EnqueueChild` rows still use `FixedFanout` and deterministic `cluster_array_index * 4` child ranges, but the buffer now contains the real hierarchy node identity beside that temporary range. The regression `node_and_cluster_cull_pass_carries_hierarchy_node_id_from_render_clusters_to_traversal_records` locks the new seam, and the cluster-work/traversal word-layout tests now prove `Some(node_id)` round-trips through the packed GPU layouts.

The newest continuation replaces that temporary child-range producer with authored hierarchy data while preserving the fixed-fanout fallback for incomplete extracts. `RenderVirtualGeometryHierarchyNode` now lives beside the other render extract DTOs, and `RenderVirtualGeometryExtract.hierarchy_nodes` carries one row per cooked hierarchy node with `instance_index`, `node_id`, `child_base`, `child_count`, `cluster_start`, and `cluster_count`. The Zircon-native CPU reference path and automatic cooked extraction both synthesize those rows from `VirtualGeometryHierarchyNodeAsset.child_node_ids`, using `child_base/child_count` as a range into the separate `RenderVirtualGeometryExtract.hierarchy_child_ids` table instead of assuming child node ids are contiguous. `virtual_geometry_node_and_cluster_cull_pass.rs` now resolves `EnqueueChild` rows by `(instance_index, hierarchy_node_id)` against `extract.hierarchy_nodes`; nodes with authored children emit `AuthoredHierarchy` plus the authored table range, while missing ids or leaf nodes still fall back to `FixedFanout`. The adjacent child-table follow-up materializes that flat child-id table as `hierarchy_child_ids + hierarchy_child_id_buffer` on the NodeAndClusterCull pass, threads it through `render.rs`, `render_frame_with_pipeline.rs`, `store_last_runtime_outputs.rs`, `scene_renderer.rs`, `new_with_icon_source.rs`, and `reset_last_runtime_outputs.rs`, and adds `read_node_and_cluster_cull_hierarchy_child_ids.rs` so renderer-owned last-state can prove non-contiguous children such as `[7, 42]` survive as authored ids. The red-green regressions `node_and_cluster_cull_pass_uses_authored_hierarchy_child_range_for_enqueue_child`, `node_and_cluster_cull_pass_points_authored_children_at_child_id_table`, `to_render_extract_carries_authored_hierarchy_child_ranges`, `to_render_extract_flattens_non_contiguous_hierarchy_child_ids`, and `node_and_cluster_cull_enqueue_child_record_roundtrips_through_gpu_word_layout` lock the new source, table semantics, and packed enum value. Focused validation stayed green with `node_and_cluster_cull_pass_`, `virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback`, `seed_backed_execution_selection_`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`.

The newest child-worklist follow-up consumes authored `EnqueueChild` traversal ranges into persistent child-node work rows instead of stopping at range markers. `VirtualGeometryNodeAndClusterCullChildWorkItem` now lives under `graphics::types` with a stable GPU word layout for parent cluster identity, optional parent hierarchy node id, child node id, child-table index, traversal index, budgets, and forced mip. `virtual_geometry_node_and_cluster_cull_pass.rs` expands only `AuthoredHierarchy` traversal records through `hierarchy_child_ids`, publishes `child_work_item_count + child_work_item_buffer`, and the render path now threads that output through `render.rs`, `render_frame_with_pipeline.rs`, `scene_renderer.rs`, `new_with_icon_source.rs`, `reset_last_runtime_outputs.rs`, and `store_last_runtime_outputs.rs` so renderer-owned last-state has the same child worklist buffer as the pass. `read_node_and_cluster_cull_child_work_items.rs` decodes the buffer back into typed rows, and `virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback` now covers the end-to-end shape by forcing one cluster to store and the second cluster to enqueue authored children `[7, 42]`. The focused regressions `node_and_cluster_cull_pass_expands_authored_child_ids_into_child_work_items` and `node_and_cluster_cull_child_work_item_roundtrips_through_gpu_word_layout` lock the pass behavior and packed layout. Focused validation stayed green with `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_expands_authored_child_ids_into_child_work_items --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_child_work_item_roundtrips_through_gpu_word_layout --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b node_and_cluster_cull_pass_ --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b seed_backed_execution_selection_ --lib -- --nocapture`, and `cargo check -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --lib`.

The newest child-decision follow-up consumes that persistent child worklist back into the traversal stream and gives child nodes their first split/keep branch. After the parent `VisitNode / StoreCluster / EnqueueChild` records are built and authored child rows are expanded, `virtual_geometry_node_and_cluster_cull_pass.rs` appends one follow-up `VisitNode` traversal record per `VirtualGeometryNodeAndClusterCullChildWorkItem`, using `hierarchy_node_id = child_node_id`, preserving the parent cluster row as the current baseline context, and resolving `node_cluster_start + node_cluster_count` through `RenderVirtualGeometryExtract.hierarchy_nodes`. `build_node_and_cluster_cull_child_decision_records(...)` now consumes those child visits: leaf or budget-kept child nodes still expand into deterministic `StoreCluster` rows, while over-budget authored child nodes with their own `child_count > 0` emit a follow-up `AuthoredHierarchy` `EnqueueChild` row carrying the child node's `child_base/child_count` table range plus the child node cluster range. The adjacent page-residency, forced-mip, and frustum follow-up threads `RenderVirtualGeometryExtract.clusters/pages`, the visit record's `forced_mip`, and the active `ViewportCameraSnapshot` into that decision producer, then filters child `StoreCluster` rows by `lod_level` when manual mip forcing is active, by positive-radius bounds-sphere camera frustum visibility, and by page residency whenever the frame carries an explicit page table; empty page tables and zero-radius bounds keep the older baseline behavior for tests and partial extracts that have not published residency or bounds yet. The newest SSE follow-up adds a pass-local baseline threshold for child-node split decisions: if any represented child cluster row has `screen_space_error` above `NODE_AND_CLUSTER_CULL_COMPAT_CHILD_SPLIT_SCREEN_SPACE_ERROR_THRESHOLD`, an authored child node with children emits `EnqueueChild(AuthoredHierarchy)` even when its cluster count is still within the current budget. This is still not full recursive BVH evaluation because the SSE threshold is local and the frustum check is a baseline camera-snapshot gate rather than a promoted `NaniteGlobalStateBuffer` contract; however, the traversal stream now distinguishes "store this mip-matching resident frustum-visible child cluster range" from "split this child node through authored hierarchy children because budget or baseline SSE says more detail is needed" instead of unconditionally storing every visited child range. The follow-up modularization moved this child-decision producer and its private store/split helpers into `virtual_geometry_node_and_cluster_cull_pass/child_decision.rs`, leaving the root pass file as the orchestration surface for launch, worklist, buffer, and test wiring while reducing the touched monolith from 2580 lines to 2385 lines. The red `node_and_cluster_cull_pass_splits_over_budget_child_nodes_into_enqueue_child_records` first failed because node `7` with range `70/3` and two authored children produced only `StoreCluster` rows; after the decision producer change it passes with `EnqueueChild(AuthoredHierarchy, cluster_array_index = 70, child_base = 1, child_count = 2)`. The newer red `node_and_cluster_cull_pass_stores_only_resident_child_cluster_pages` then failed with `[70, 71]` because child cluster `71` lived on a nonresident page, and now passes with only `[70]`. The forced-mip regression `node_and_cluster_cull_pass_stores_only_forced_mip_child_clusters` likewise failed with `[70, 71]` before the child-store gate read `forced_mip`, and now keeps only cluster `70`. The frustum regression `node_and_cluster_cull_pass_stores_only_frustum_visible_child_clusters` first failed with `[70, 71]`, then passes with only the in-frustum child cluster `70`; the SSE regression `node_and_cluster_cull_pass_splits_child_nodes_when_cluster_sse_exceeds_threshold` first failed because the budget-kept child node stored its cluster immediately, then passed by emitting the same authored child split record from the over-threshold cluster's `screen_space_error`. The existing `node_and_cluster_cull_pass_consumes_child_work_items_into_child_visit_records` still proves leaf child nodes expand into `StoreCluster(70/71/72/90)` when no page table and no positive-radius bounds are present. Focused validation for these slices used `cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation node_and_cluster_cull_pass_splits_child_nodes_when_cluster_sse_exceeds_threshold --lib -- --nocapture`, `cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation node_and_cluster_cull_pass_stores_only_frustum_visible_child_clusters --lib`, `cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation node_and_cluster_cull_pass_ --lib`, `cargo fmt --check --package zircon_runtime`, and `cargo check -p zircon_runtime --lib --locked --target-dir target/manual-physics-animation`.

The newest structural follow-up splits the overgrown `virtual_geometry_node_and_cluster_cull_pass.rs` production body into responsibility-backed submodules without changing the public pass output. `startup_worklist.rs` now owns global-state, dispatch setup, launch worklist, instance work item, and cluster work item construction; `traversal.rs` owns the parent `VisitNode / StoreCluster / EnqueueChild` traversal records plus authored-hierarchy fallback ranges; `child_worklist.rs` owns authored child-id table expansion and child `VisitNode` row construction; `child_decision.rs` owns the budget, forced-mip, page-residency, frustum, and baseline-SSE child split/store branch; and `buffers.rs` owns renderer-owned GPU buffer materialization. The root file still carries the inline regression suite, so it remains above the preferred size threshold, but the production path is now an orchestration layer with clear next extraction boundaries rather than one mixed implementation file.

The latest code-first follow-up promotes the child split/store policy knobs into `RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot`: `child_split_screen_space_error_threshold` and `child_frustum_culling_enabled` now round-trip through the packed global-state layout and are consumed directly by `child_decision.rs`. The instance-work-item WGSL launch header offset was updated with the wider global-state layout, so renderer-owned readback still decodes the same typed per-instance work rows. This keeps the current baseline defaults while moving them onto the future Nanite global-state seam.

The newest code-first follow-up feeds child-decision `EnqueueChild(AuthoredHierarchy)` output into bounded follow-up traversal waves. `virtual_geometry_node_and_cluster_cull_pass.rs` now repeats child-work expansion, child visit resolution, and child decision generation for newly emitted authored child ranges up to a baseline wave cap, so a split child node can immediately visit and store its authored grandchild cluster range instead of stopping after the first decision layer.

The latest code-first follow-up gives that child decision path its first pass-local page feedback. A visible, forced-mip-compatible child cluster whose page is not resident is no longer only skipped; `child_decision.rs` records the missing `page_id`, `buffers.rs` materializes a `page_request_buffer`, and the render path preserves `page_request_count/page_request_ids/page_request_buffer` through renderer last-state. This remains request feedback only; runtime upload authority still belongs to the existing VG residency host.

The public debug-snapshot word-stream contract now exports both `RenderVirtualGeometryNodeAndClusterCullWordStreams` and `RenderVirtualGeometryNodeAndClusterCullDecodedStreams` from `core::framework::render`. Tooling can request packed node-and-cluster-cull streams for GPU-layout parity, then call `RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_word_streams(...)` to recover the typed global-state, dispatch, launch-worklist, work-item, traversal, hierarchy-child, and page-request DTOs without depending on renderer-private readback helpers.

The readback stream contract now also exposes full-chain raw, decoded, footprint, report, summary, and diagnostic decode surfaces from `virtual_geometry_debug_snapshot_streams.rs`. `debug_readback_streams()` groups NodeAndClusterCull, render-path, and VisBuffer64 streams; `debug_readback_stream_footprint()` reports raw u32-word and byte totals without decoding, so malformed payloads can still be budgeted; `debug_readback_stream_report()` packages that footprint with either a decoded summary or the first section decode error; `debug_decoded_streams()` and `debug_readback_stream_summary()` preserve optional decode paths; and the `try_*` decode/summary methods return explicit section errors such as malformed NodeAndClusterCull child-work words, malformed render-path selected-cluster words, or invalid VisBuffer64 packed state bits while leaving renderer-private helpers out of tooling code. Stream, footprint, summary, and report helpers now also distinguish real payload from the mandatory VisBuffer64 clear word via `has_payload()`, `is_empty()`, `payload_u32_word_count()`, `payload_byte_count()`, `section_u32_word_count()`, `section_payload_u32_word_count()`, `decode_error_section_u32_word_count()`, `is_decodable()`, and `has_decoded_payload()`, with those metric/count impls isolated in `virtual_geometry_debug_snapshot_streams/metrics.rs` so empty-frame tooling can remain byte-budget aware without reporting false readback content. Decode-error ownership and stride-failure word counts are queryable through `section()`, `decode_error_section()`, `malformed_u32_word_count()`, and `decode_diagnostic()`, with those diagnostics isolated in `virtual_geometry_debug_snapshot_streams/diagnostics.rs`.

The newest test-structure follow-up moves that remaining inline regression suite into `virtual_geometry_node_and_cluster_cull_pass/tests/`. `tests/mod.rs` is now wiring only, `prelude.rs` centralizes shared imports, `support.rs` owns the offscreen-frame/pass helpers, and the behavioral coverage is split across `startup.rs`, `traversal.rs`, `hierarchy.rs`, and `child_decision.rs`. This keeps the production root at orchestration size and keeps future NodeAndClusterCull tests near the behavior they exercise instead of rebuilding another thousand-line inline test block.

The latest plugin-boundary follow-up also turns `VirtualGeometryNodeAndClusterCullPassOutput` into an owner-private pass-output seam. `output.rs` hides the wide startup, worklist, traversal, child-work, hierarchy-child, and page-request field layout; `execute.rs` constructs the output only through `VirtualGeometryNodeAndClusterCullPassStoreParts`; seed-backed executed-cluster selection reads the pass through `source()`, `instance_work_items()`, and `cluster_work_items()`; and renderer last-output storage consumes `into_store_parts()` instead of reconstructing the field names. The broad pass regressions now use `#[cfg(test)]` accessors and explicit clear helpers for contract-break cases, so production code no longer depends on the pass DTO layout while a future VG plugin runtime can replace the producer behind the same store-parts handoff.

The Runtime UI drag-source validation pass on 2026-04-28 exposed two adjacent compile-boundary drifts before the editor tests could link: the builtin render-feature dispatcher was still importing `RenderFeatureCapabilityRequirement` through an obsolete sibling path, and the split `seed_backed_execution_selection` submodules were re-exporting helpers to the parent executed-cluster pass without matching item visibility. The dispatcher now imports the capability requirement from the public `graphics::feature` surface, while `collect_execution_cluster_selection_collection_from_root_seeds`, `seed_backed_cluster_ordering`, and `SeedBackedClusterOrdering` are scoped only as far as `virtual_geometry_executed_cluster_selection_pass`; ordering fields remain local to the seed-backed module. A follow-up compile attempt reached the Runtime crate again and then became blocked by active shared Cargo/package-cache locks, so the boundary fix is formatted but still awaiting a quiet Cargo rerun for full editor drag-source verification.

The latest plugin-boundary cleanup also makes `VirtualGeometryGpuResources::new(...)` folder-backed. The owner declaration still lives in `virtual_geometry_gpu_resources.rs`, while `new/mod.rs` only orchestrates construction; uploader bind-group layout, uploader pipeline creation, params buffer allocation, and node-and-cluster-cull instance-work-item pipeline creation now live in separate child modules. This keeps the heavy WGPU resource bootstrap movable as one resource package when the VG runtime plugin takes ownership, without leaving a migration-only `new.rs` forwarding file behind.

The latest GPU resource-owner cleanup also moves the node-and-cluster-cull instance-work-item compute dispatch behind `VirtualGeometryGpuResources::create_node_and_cluster_cull_instance_work_item_buffer(...)`. The render pass receives only `SceneRendererAdvancedPluginResources` and calls its narrow VG cull buffer facade; it no longer imports `VirtualGeometryGpuResources` or reads the bind-group layout / compute pipeline fields. Those two WGPU handles are now package-local to the VG resource owner, so the future plugin runtime can replace the resource package without preserving a raw renderer-side resource view.

The descriptor-gated resource allocation follow-up keeps this owner boundary but stops constructing VG GPU resources for a base renderer that was created without linked VG render descriptors. `SceneRendererAdvancedPluginResources` now creates `VirtualGeometryGpuResources` and `VirtualGeometryIndirectArgsGpuResources` only when the linked descriptor set contains the `VirtualGeometry` capability requirement. Mesh draw building therefore receives an optional indirect-args resource and builds the shared VG indirect buffer family only when both the compiled frame enables VG and the renderer actually owns the linked VG resource package.

## Next Expected Layers

The next Nanite plan steps should build on this foundation instead of redefining it:

- a real VG cook pipeline that fills `virtual_geometry`
- scene extraction that sources live VG instances from cooked assets
- runtime-page integration between the CPU reference output and the existing VG residency host
- recursive child-node split/keep evaluation that turns the current global-state baseline-SSE/frustum knobs into camera-derived policy, then connects pass-local page requests into runtime upload authority
- `VisBuffer64`, full `NodeAndClusterCull`, and hardware raster passes
- automatic SSE-driven LOD and multi-pass hierarchy culling
