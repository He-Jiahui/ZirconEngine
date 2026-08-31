---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/access_bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/external_access_bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/persistent_texture_access_bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/lifecycle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/reporting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/texture_views.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_device_epoch_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/native.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_command_lists.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/terminal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/domain_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/reports.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/light_scatter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/generic_compute_executor.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors/resolve_trace_handoff.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/history_epilogue_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/access_bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/persistent_texture_access_bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/lifecycle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/reporting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/texture_views.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_device_epoch_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/native.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_command_lists.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/terminal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-14-render-owner-budget-splits.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
  - user: 2026-06-17 implement transient pool budget/eviction for render plan 01
  - user: 2026-06-17 implement RG-M4 alias map/profile timings for render plan 01
  - user: 2026-06-17 implement graph materialization validation for render plan 01
  - user: 2026-06-17 bind HZB executor-owned external buffers for render plan 01
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
  - user: 2026-06-17 continue Plan 01 required external texture import and materialization modularization
  - user: 2026-07-06 implement WGPU-to-render-pipeline Plan 11 IBL bake storage view bridge
  - user: 2026-07-08 continue WGPU-to-render-pipeline Plan 18 Hybrid GI graph scene-depth MSAA handoff
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs::registry_rejects_unregistered_executor_ids
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs::taa_reactive_mask_clear_executor_requires_graph_resources_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs::screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pass_executor_registry_tests.rs::runtime_15_render_pass_executor_registry_tests_are_child_owners
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pass_gpu_context_mesh_command_lists.rs::runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_aliases_compatible_transient_texture_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_receives_incompatible_texture_resources_in_separate_graph_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_aliases_transient_buffer_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_reuses_entries_across_frames
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_evicts_stale_entries_after_keep_frames
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_evicts_oldest_entries_to_budget
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::render_post_dynamic_resolution_scale_swap_releases_pool
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_creates_dense_transients_and_skips_sparse_reservations
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_exposes_owned_cube_storage_texture_array_views
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_reports_unbound_compiled_lifetimes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_reports_unbound_external_lifetimes_without_failing
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_reports_unbound_typed_optional_external_without_failing
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_fails_unbound_required_external_buffer
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_fails_unbound_required_external_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_rejects_stale_texture_binding_outside_live_lifetimes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_rejects_stale_buffer_binding_outside_live_lifetimes
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_preserves_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_rejects_conflicting_required_external_texture_and_buffer_binding
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_imports_only_live_compiled_frame_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_rebinds_live_final_aliases_to_imported_texture_target
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_leaves_advanced_transients_for_materialization
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs::tests::compiled_scene_graph_stage_lists_keep_early_and_late_boundaries
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs::tests::active_late_graph_stages_follow_compiled_pipeline_order
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs::tests::compiled_scene_sprite_stage_list_owns_core2d_product_stages
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs::tests::active_sprite_graph_stages_follow_unculled_sprite_passes
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_forward_plus_preserves_shadow_atlas_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_deferred_preserves_shadow_atlas_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs::compile_preserves_report_only_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs::compile_preserves_report_only_external_buffer_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs::compile_rejects_conflicting_report_only_external_texture_and_buffer_binding
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs::tests::rg_resource_resolver_requires_pass_declared_access_before_physical_texture_lookup
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs::tests::resolver_backed_name_access_ignores_stale_context_resource_rows
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs::tests::public_gpu_buffer_lookup_requires_compiled_pass_declaration_access
  - cargo test --manifest-path zircon_runtime\Cargo.toml public_gpu_buffer_lookup_requires_compiled_pass_declaration_access --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hgi-msaa-depth-handoff-0708 --message-format short --color never -- --nocapture --test-threads=1
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs::tests::hzb_external_fallback_buffers_satisfy_materialization_report
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs::tests::hzb_occlusion_cull_declares_execution_owned_external_buffers
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_describes_hzb_as_half_power_of_two_mip_chain
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs::tests::particle_runtime_prepare_neutral_frame_sizes_cover_readback_payload
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs::tests::particle_runtime_prepare_neutral_frame_uses_minimum_nonzero_buffers
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs::tests::particle_runtime_prepare_registration_id_is_stable
  - zircon_plugins/particles/runtime/src/tests/manager_resolution.rs::particles_runtime_plugin_module_and_runtime_prepare_share_manager
  - zircon_plugins/particles/runtime/src/tests/gpu.rs::particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers
  - cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-alias-profile-0617
  - cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-materialization-0617
  - cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-materialization-0617
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-binding-contract-0617 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-required-external-texture-0617 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-typed-optional-external-0617 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-resolver-cutover-0617 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-resolver-cutover-0617-hzb-shadow-velocity --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-f16-0622-coremin --message-format short --color never
  - cargo test -p zircon_runtime --lib active_late_graph_stages_follow_compiled_pipeline_order --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-f16-0622-coremin --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib active_sprite_graph_stages_follow_unculled_sprite_passes --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-f16-0622-coremin --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib compiled_scene --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-f16-0622-coremin --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib materialization_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib materialization_exposes_owned_cube_storage_texture_array_views --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-command-plan-0706 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# RenderGraph Execution Resources

`RenderGraphExecutionResources` is the WGPU-side resource table used by scene renderer graph executors. The compiled `RenderGraph` remains RHI-neutral and exposes logical resource declarations, lifetimes, the transient allocation plan, and an immutable external-access packet; this module turns live lifetimes into concrete WGPU objects while retaining name lookup only at the compatibility boundary.

The resource table is folder-backed by ownership: `mod.rs` only mounts the module and owns the
per-frame state; `binding.rs` owns logical-to-physical registration, `lookup.rs` owns pass-facing
resolution, `lifecycle.rs` owns pooled materialization and release, `reporting.rs` owns diagnostics,
and `texture_views.rs` owns view-descriptor validation; `access_bindings.rs` owns exact transient
views, buffer windows, and one WGPU texture backing per compiler physical allocation;
`external_access_bindings.rs` owns the access-ID keyed physical external lease table; and
`persistent_texture_access_bindings.rs` owns exact persistent texture/backing/view leases. This is a hard cut from the former flat
owner, so consumers retain the selected `RenderGraphExecutionResources` surface without a
compatibility module.

## Logical Names And Backing Resources

Executors still ask for resources by logical names such as `scene-color`, `screen-space-reflection-depth-pyramid`, or `light-list`. Internally, the execution resource table now has an additional mapping from logical names to physical backing names. That lets two non-overlapping graph resources share one owned WGPU backing while preserving the existing executor API:

- `imported_texture_views` maps every logical texture name to the view an executor uses.
- `owned_textures` and `owned_texture_descs` store physical WGPU texture backings.
- `owned_texture_backings` maps logical texture names to the physical backing key.
- `buffers` stores physical WGPU buffer backings.
- `buffer_backings` maps logical buffer names to the physical backing key.

This keeps the graph-execution-local `require_texture_view(...)`, `require_buffer(...)`, and `owned_texture(...)` lookup shape stable while allowing the materialization step to use the compiled transient plan. The raw texture/buffer lookup helpers are not part of the public scene-renderer surface; pass executors should resolve through `RgResourceResolver` or the GPU context helper layer. The three compatibility mutators that import a raw view, insert a raw buffer, or create a texture alias are scoped to the scene-renderer owner as well; plugin and external feature code must use the qualified binding packet or the graph resource binder instead of mutating this table directly.

The GPU execution context follows the same boundary for native command recording. Its `Device`,
`CommandEncoder`, and scene bind handles are stored with scene-renderer visibility; cross-crate
features must request the short-lived `RenderPassGpuNativeContext` capability through
`native_context()`. The capability intentionally excludes the graph resource table and plugin
output mailbox, while readback publication uses the explicit `plugin_outputs()`/
`plugin_outputs_mut()` methods. The native context does not expose its raw `Device`: pass-local
buffer, bind-group, layout, shader-module, and pipeline creation goes through
`RenderPassGpuResourceFactory`. This keeps graph texture allocation outside the pass capability and
counts every admitted create in the current pass profile without changing encoder or submission
ownership.

## External Access Leases

`CompiledRenderGraph::external_access_packet()` contains one immutable entry for each live external
access: its stable `RenderGraphResourceAccessId`, versioned key, declared external type, and any
producer-supplied physical descriptor. After frame and plugin producers bind their WGPU objects,
`materialize_external_access_bindings(...)` creates an access-ID keyed lease table. Generic compute
and resolver-backed non-compute consumers resolve typed external textures and buffers through this
table, while indirect-dispatch arguments use the same exact buffer lease. A concrete buffer access
scope must agree with the schema window; descriptor-less report-only imports remain compatible on
the legacy name path, but a pipeline that requires a physical texture descriptor fails closed
rather than guessing from a resource label.

## Pass-Scoped Resolver Lookup

`RgResourceResolver` owns the pass-scoped declaration/access check that sits between compiled graph metadata and physical WGPU lookup. `RenderPassExecutionContext::require_texture_view_by_name(...)` first asks the resolver to prove that the current compiled pass declared the requested resource name and access kind, then resolves the physical view through `RenderGraphExecutionResources` by graph declaration instead of trusting a copied context row. The resolver also carries physical texture/buffer lookup helpers for the remaining executor migrations.

`CompiledRenderGraph` builds typed-resource and name declaration indices, a pass-ID index, and a `(pass, resource, access)` index once during compilation. Resolver checks therefore select the current pass, declaration, and declared access in expected O(1) time instead of repeatedly scanning the pass table and each pass resource list. Name-based lifetime lookup also resolves through the declaration and typed lifetime indices. The vectors remain the canonical ordered storage used by dumps and public iteration; the private maps only index those vectors and preserve the first declared access when a malformed input repeats the same key. `rg_resource_resolver_materialization_indices_follow_topologically_reordered_passes` covers the important case where stable pass IDs do not match topologically compiled vector positions.

The 2026-07-07 Hybrid GI graph scene-depth handoff made the public GPU context buffer lookup explicit.
`RenderPassGpuExecutionContext::require_buffer(...)` now mirrors texture-view lookup by checking the
compiled pass declaration through `RgResourceResolver` before returning a WGPU buffer. The first
consumer is the HGI `hybrid-gi-scene-prepare` graph pass: it reads the pass-declared `scene-depth`
texture view and writes the pass-declared `hybrid-gi-scene` buffer from a WGPU compute pass.

The 2026-07-08 Hybrid GI MSAA follow-up adds descriptor lookup to the same resolver-backed path.
`RenderGraphExecutionResources::require_texture_desc_for_declaration(...)` first returns the
compiled graph declaration's `RenderGraphResourceDesc::Texture` descriptor, which covers
frame-imported resources such as fixed `scene-depth`; untyped external resources fall back to
`require_owned_texture_desc(...)`, which resolves a logical transient through its physical owned
backing. `RenderPassGpuExecutionContext::require_texture_desc(...)` exposes the resulting cloned
`TextureDesc` only after the current pass has declared the requested texture access, so executors can
select shader variants from graph texture metadata without bypassing the compiled pass contract. The
focused runtime regression now builds a 4x MSAA `scene-depth` texture, proves the descriptor sample
count is visible to the declared read pass, proves the declared `hybrid-gi-scene` write buffer is
accessible, and still rejects undeclared buffer read access. The command
`cargo test --manifest-path zircon_runtime\Cargo.toml public_gpu_buffer_lookup_requires_compiled_pass_declaration_access --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hgi-msaa-depth-handoff-0708 --message-format short --color never -- --nocapture --test-threads=1`
passed 1/1.

The first production cutover is deliberately narrow. Product post-process executor validation and terminal input/output selection now use resolver-backed name access, so stale `context.resources` rows copied into an execution context cannot authorize a post-process input. `RenderPassExecutionContext::with_gpu(...)` now propagates the same pass resolver into `RenderPassGpuExecutionContext`. Depth-prepass normal/depth writes, deferred G-buffer albedo/material writes plus depth reads, deferred lighting G-buffer/light-grid/background/scene-color lookups, shadow-atlas writes, HZB previous-history reads, mesh-stage color/depth attachments, mesh light-grid buffer reads, object and particle velocity attachments, TAA reactive-mask mesh color/depth attachments, sprite/preview-sky/UI/overlay surface bridge lookups, particle transparent bridge lookups, SSR-specific resolve/reflection-pyramid/coarse-pyramid/specular-occlusion bridge lookups, and root postprocess stack/effect/compute/terminal bridge lookups now resolve through `gpu/resource_lookup.rs` helpers after checking the compiled pass access declaration. Mesh light-grid buffers and optional shared post-process texture slots use optional resolver-backed helpers so passes that declare a resource fail on kind mismatches while shared executor variants that do not declare that resource keep their fallback texture behavior. HZB history uses a declared-optional helper: the compiled pass must declare the history read, but the first-frame physical texture may still fall back to the post-process white texture. The deferred scene bridge methods live in `gpu/deferred.rs`; the sprite, screen-space UI, preview-sky, and overlay bridge methods live in `gpu/surface.rs`; particle bridge methods live in `gpu/particle.rs`; root postprocess orchestration remains in `gpu/post_process.rs`; effect-chain, compute-resource, temporal, terminal, and SSR bridge methods live in `gpu/post_process/effects.rs`, `computed_resources.rs`, `temporal.rs`, `terminal.rs`, and `screen_space_reflection.rs`. The particles plugin executor no longer performs its own direct WGPU texture-view precheck before entering the runtime particle bridge. SSR passes also declare `light-list` as an external buffer read before the shared post-process bind group is recorded. Direct execution-resource lookup in GPU executor code is now confined to `gpu/resource_lookup.rs` helper fallback/internal calls; the remaining Plan 01 resolver hard-cutover work is non-HZB/non-shadow-atlas external ownership, focused resolver tests, and the larger structural split needed if raw physical lookups must be isolated beyond graph-execution scope.

The Plan 01 GPU context mesh command lists owner split (`render_plan01_gpu_context_mesh_command_lists_owner_split_static_passed_cargo_deferred_active_compile_lane`) keeps `gpu.rs` focused on `RenderPassGpuExecutionContext` construction and context access, `gpu/mesh_command_lists.rs` owns `RenderPassMeshCommandLists`, per-phase streams and HZB counters, and `gpu/mesh_recording.rs` owns depth-prepass, shadow-atlas, standard/advanced/transmission mesh-stage and TAA reactive-mask recording. Guard `runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner` locks all three owners below 800 lines; the Runtime15 follow-up status is `runtime_15_render_owner_budget_split_current_source_managed_build_passed`.

## Alias And Profile Reports

After graph targets, optional history resources, and transient resources are materialized, `RenderGraphExecutionResources::resource_alias_report()` snapshots the logical-to-physical map before the frame releases its owned backings into the cross-frame pool. The report is framework-neutral: `RenderGraphExecutionAliasRecord` stores only a logical resource name and a physical backing label. It covers owned-backed graph resources, transient slot sharing such as `scene-color -> rg-transient-texture-bucket-<hash>-slot-0`, transient buffer sharing with the same bucketed label shape, SSR mip views as `parent:mipN` aliases, and execution-owned external buffer aliases such as `mesh.indirect-args -> mesh.indirect-args:hzb-execution-phase0`. Externally imported frame targets are intentionally excluded because the execution table owns only cloned views for those resources, not their source texture identity.

`RenderGraphExecutionRecord` carries this alias report beside the existing `RenderGraphExecutionResourceReport`. `update_base_stats(...)` copies it into `RenderStats.last_graph_execution_alias_report`, and runtime diagnostics only project stable counts: logical texture/buffer names, aliased names, and distinct physical backing labels. The full alias rows stay available to query/capture consumers without creating high-cardinality diagnostic paths.

Pass profile timing follows the same execution-record path. `execute_graph_stage(...)` times each executor call with a CPU `Instant`, records the pass identity, budget, CPU time, render/dispatch work, upload bytes, and `RenderPassNativeResourceCreateMetrics`, then exposes the per-frame rows through `RenderGraphExecutionProfileReport`. The native create metrics distinguish buffers, bind groups, bind-group layouts, shader modules, pipeline layouts, compute pipelines, and render pipelines. `RenderFrameProfile` preserves those per-pass rows; fixed-cardinality diagnostics aggregate them under `render.profile.native_resource_create.*` without creating pass-name paths. Compute workload audit context uses immutable frame geometry plus the volumetric source descriptor frozen in `CompiledHistoryEpiloguePlan`; it does not recover the final froxel output by resource name from the materialized table. CPU spans and resource-create counts are measurement inputs, not GPU timestamp queries or proof that a cache will improve performance; GPU timestamp and RenderDoc profile alignment remain future profiling work.

The first built-in coverage cut is the generic compute executor. Its bounded pipeline cache keeps the
raw device only for limits and validation error scopes; cache-miss bind-group layout, pipeline-layout,
shader-module, and compute-pipeline creates use the pass factory, as does the dispatch bind group.
A successful cache hit therefore contributes one observed create and a successful cold miss five.
Other built-in renderer helpers that still receive the owner-scoped raw device are not yet covered,
so the metric is explicitly factory-admitted work rather than a claim of complete WGPU allocation
interception.

## Materialization Completeness Report

`RenderGraphExecutionResources::validate_materialized_graph_resources(...)` now delegates to `graph_execution/materialization_validation.rs`, keeping the execution resource table focused on WGPU backings while the validation module owns lifetime auditing. The audit runs after WGPU materialization and before pass execution. Dense texture lifetimes must resolve to a texture view, dense buffer lifetimes must resolve to a buffer, and sparse texture reservations are counted without requiring dense backing. Missing typed texture or buffer bindings return a `GraphicsError::Asset` through the compiled-scene render path before any executor can hide the gap behind a private lookup.

The audit also separates hard-required external coverage from report-only external coverage. External lifetimes carry `RenderGraphExternalResourceBinding`, so the report records `required_external_count`, `bound_required_external_count`, and `missing_required_external_count` only for imports declared through `required_buffer()` or `required_texture()`. Report-only unknown/texture/buffer imports are counted through `report_only_external_count`, `bound_report_only_external_count`, and `missing_report_only_external_count`, so frame targets, history inputs, optional post-process resources, and first-party plugin resources keep diagnostic type intent without being mislabeled as required. The aggregate `bound_external_count()` and `missing_external_count()` methods still provide total external coverage for diagnostics. Missing required external lifetimes fail before pass execution, while missing report-only lifetimes contribute to external misses without failing `materialized_resources_complete()`. `RenderFeaturePassDescriptor` exposes typed optional helpers and required helpers, and `RenderPipelineAsset` preserves the external binding on compiled external lifetimes while rejecting conflicting external texture/buffer declarations. `RenderGraphMaterializationReport::materialized_resources_complete()` therefore means dense typed texture/buffer materialization is complete, while `is_complete()` includes both required and report-only external coverage.

The same validation pass treats compiled lifetimes as the authoritative live set. Any logical texture view or buffer binding already present in `RenderGraphExecutionResources` but absent from `CompiledRenderGraph::resource_lifetimes()` is rejected as a stale binding before executor dispatch. This catches frame, history, plugin, or fallback binders that accidentally pre-bind resources from culled graph paths. The report exposes `stale_texture_binding_count`, `stale_buffer_binding_count`, and aggregate `stale_binding_count()`, and both `materialized_resources_complete()` and `is_complete()` require those counts to stay zero.

HZB occlusion now declares those executor-owned resources as required external buffers and binds them into the same table before validation. `bind_execution_owned_graph_resources(...)` inspects live graph lifetimes and phase-local mesh indirect executions, then registers source indirect args, compaction metadata, compacted args, visible-instance remap, indirect draw-count, and HZB stats buffers through `bind_execution_owned_buffer(...)`. Empty or disabled phase lists get minimum bindable fallback buffers so a compiled HZB occlusion pass still reports bound external lifetimes instead of hiding the gap behind executor-private state.

Renderer-owned frame resources now have an explicit actual-binding owner in `render/bind_frame_graph_resources.rs`. The binder imports only fixed frame targets with live compiled lifetimes: scene color/depth, final-target aliases, G-buffer/lighting frame views, `LIGHT_LIST`, and the persistent `SHADOW_ATLAS` view. `SHADOW_ATLAS` remains the first production required external texture; the compiled Forward+ and Deferred graphs preserve one required external texture lifetime for the `shadow-atlas` writer plus mesh/deferred receiver readers, and the binder maps that lifetime to `ShadowAtlasResources::atlas_view()` before materialization validation. Missing atlas import is therefore still a hard validation error, while unused frame targets stay absent from the execution table. As of the 2026-06-19 Plan 09 post-process viewport slice, `postprocess.bloom` and `postprocess.global-illumination` are no longer prebound fixed frame targets: the post-process graph declares them as owned local textures so selected-camera split viewport execution can keep local coordinates for intermediates and only translate when sampling fixed full-frame sources or writing terminal output.

The 2026-06-22 F16 structure slice split the compiled-scene render orchestration without changing graph semantics. `render/bind_compiled_scene_graph_resources.rs` now composes frame imports, history imports, transient materialization, execution-owned HZB buffers, and plugin runtime-prepare external buffers before materialization validation. The narrower `bind_frame_graph_resources.rs`, `bind_history_graph_resources.rs`, `bind_execution_owned_graph_resources.rs`, and `bind_plugin_graph_resources.rs` remain the actual owner modules for each binding family. `render/execute_compiled_scene_graph_stages.rs` now owns the early graph stages, optional lighting stage, scene passes, post-process stage, history-copy report, and late UI/overlay/debug stage loop, including the associated RenderDoc marker scopes and stage-order tests. `render/submit_compiled_scene_frame.rs` owns final command submission, HZB indirect-args readbacks, HZB cull readback reporting, test-only scene-velocity/exposure/color-LUT readbacks, and the transient-pool release/report update. `render.rs` remains the compiled-scene orchestration boundary for draw preparation, graph binding/stage/submission delegation, renderer output assembly, and GPUScene previous-frame rolls.

Built-in history resources now have an explicit actual-binding owner in `render/bind_history_graph_resources.rs`. The render path computes frame-level availability for TAA scene color, screen-space reflection, HZB, Hybrid GI, and exposure history, then the binder imports only enabled resources that are live in the compiled graph. It binds TAA previous/current texture views, previous SSR/HZB texture views, the `history-global-illumination` alias, and exposure previous/current buffers before transient materialization validation. This keeps history imports graph-lifetime-aware without treating those history textures as plugin-owned resources. Hybrid GI history writeback is paired with the graph-owned GI output: `copy_history_textures.rs` copies the owned `postprocess.global-illumination` texture into the selected camera's physical history region when present, falling back to the legacy fixed target only if that graph output is unavailable.

TAA scene color and exposure use a different ownership path from epilogue texture copies: their current history resources are external graph destinations written directly by the TAA attachment pass and exposure compute pass. `RenderPassGpuExecutionContext` records a `SceneHistoryWriteIntent` only after the corresponding command encoding succeeds. SSR resolve, HZB build, volumetric light-scatter, generic-compute SSAO, and the Hybrid GI plugin resolve publish the same receipt after producing their graph-owned or external copy source. The plugin-facing entry is `record_frame_history_write(FrameHistorySlot)`, which maps the public compiled-feature history contract onto the renderer's scene-history domains without exposing the private transaction type. `RecordedGraphPass` carries these receipts across serial or parallel recording, and `RenderGraphStageExecution` merges committed pass receipts for the frame. The history epilogue uses that merged receipt when producing `RenderHistoryCopyReport` and the frame transaction, so a merely declared live writer cannot validate direct history or copy a stale GI/AO/SSR/HZB/volumetric source when its executor skipped or failed. Submission remains the only persistent-history commit point.

First-party plugin external buffers now have a separate graph-lifetime-aware binding owner in `render/bind_plugin_graph_resources.rs`. Runtime prepare collectors can register actual per-frame WGPU buffers through `RuntimePrepareCollectorContext`, and `SceneRendererAdvancedPluginReadbacks` carries those bindings to the graph binder. Registered buffers are bound first and produce runtime-prepare alias rows; the current particles GPU buffer names and `virtual-geometry-feedback` still receive deterministic `:plugin-external-fallback` backings when no registered buffer is present. Virtual Geometry now registers a runtime-prepare backing for prepared NodeAndClusterCull page-request feedback, so live `virtual-geometry-feedback` lifetimes use that sideband buffer when page requests exist. Particles now register `particles.runtime-prepare` with the plugin's shared `ParticlesManager`: concrete GPU instances execute through `ParticleGpuRuntimeOwner` and register real `ParticleGpuBackend` buffers for `particles.gpu.*`; frames without concrete GPU instances can still use neutral `ParticleExtract.gpu_frame` summary-derived buffers, and frames without either source fall back to deterministic materialization buffers.

`RenderGraphExecutionRecord` carries this report beside the resource, alias, and profile reports. `update_base_stats(...)` copies it into `RenderStats.last_graph_materialization_report`, and diagnostics mirror stable counts under `render.graph.materialization.*`, including total required/bound/missing resources, missing typed resources, texture/buffer splits, aggregate external coverage, required-external coverage, report-only external coverage, stale logical binding counts, and sparse texture reservation count.

## Transient Slot Materialization

`materialize_transient_resources_with_pool(...)` delegates transient slot lowering to `graph_execution/transient_materialization.rs` and consumes `CompiledRenderGraph::transient_allocation_plan()` instead of allocating every dense logical resource independently. The pool argument is mandatory: the former test-only unpooled entry point has been removed, and the internal materialization chain no longer accepts `Option<&mut TransientResourcePool>`. Texture and buffer allocations are grouped by descriptor bucket plus bucket-local graph slot:

- Dense texture slots create one WGPU texture backing after the graph planner has bucketed them by dimensions, mip levels, array depth, sample count, format, dimension, residency, and usage. Execution still checks those fields defensively before sharing a backing.
- Dense buffer slots create one WGPU buffer backing after the graph planner has bucketed them by size and usage.
- Sparse texture reservations stay unbacked by dense WGPU resources. They remain visible through graph lifetimes and stats only.

The RenderGraph allocation plan is therefore the neutral descriptor-bucketed aliasing contract, while this module enforces the stricter WGPU object-compatibility rules at execution time. Materialization keys physical backings by `(bucket_key_hash, slot)` and emits bucketed backing labels, so two descriptor buckets that both own slot `0` stay separate in the WGPU resource table and alias report. Graph dump transient slot rows include `bucket_key_hash`, and resource rows report bucket-local reserved bytes through `slot_bytes_for_bucket(...)`.

`materialization.rs` now stays focused on the execution materialization entry point, SSR alias attachment, and WGPU descriptor conversion helpers shared with `TransientResourcePool`. The 2026-06-24 RenderGraph materialization test owner split moves storage-usage, dense/sparse transient materialization, aliasing, terminal AA override, mip-view, and SSR coarse-pyramid alias tests into `graphics/scene/scene_renderer/graph_execution/materialization/tests.rs`; guard `runtime_15_render_graph_materialization_tests_are_child_owner_split` locks the split under `render_plan01_materialization_tests_owner_split_static_passed_cargo_deferred_active_compile_lane`. `RenderGraphExecutionResources` remains the logical-to-physical table and reporting surface; it no longer owns transient slot grouping logic.

## Cross-Frame Pool

`SceneRendererCore` owns a `TransientResourcePool` for WGPU physical resources. A render starts the pool frame before graph materialization, materializes logical graph resources through the pool, submits the command encoder, then releases all owned graph backings into the pool and ends the pool frame. Pool keys include the WGPU-relevant descriptor shape and usage bits, so a texture or buffer is reused only when the next frame requests a compatible backing. Stale entries are evicted after `TRANSIENT_RESOURCE_POOL_KEEP_FRAMES` pool frames.

This is also the only materialization contract used by WGPU-facing tests. Each fixture creates and begins an explicit pool before calling `materialize_transient_resources_with_pool(...)`; a pool miss may create a new WGPU object, but no materialization caller can bypass pool accounting or descriptor-key policy. Runtime15 guard `runtime_15_render_graph_materialization_requires_transient_pool` rejects restoration of either the unpooled execution-resource method or optional pool plumbing.

The pool is now byte-budgeted in addition to frame-age bounded. Returned textures store `TextureDesc::checked_storage_size_bytes()` as their estimated retained size, returned buffers store `BufferDesc.size_bytes`, and frame end first removes stale entries, then evicts the least-recently-used retained entries until the texture and buffer pools fit their independent budgets. The default internal budgets are `TRANSIENT_RESOURCE_POOL_TEXTURE_BUDGET_BYTES` and `TRANSIENT_RESOURCE_POOL_BUFFER_BUDGET_BYTES`; tests can inject smaller budgets to exercise the eviction path without allocating large GPU resources.

This preserves the existing per-pass resolver contract while adding the RDG-style distinction between logical graph resources and reusable physical resources. The current implementation still binds all live resources for the frame up front; it does not do pass-boundary acquire/release inside a command encoder. Both abort release and submitted retirement clear the exact-access tables before returning allocations to the pool, so the frame-scoped full-texture handles cannot extend the physical owner lifetime. Budget eviction only runs after the frame's owned backings have been released, so it never removes resources that are still bound by the in-flight graph execution table.

The pool publishes `RenderGraphTransientPoolReport` through `RenderGraphExecutionResourceReport`. Runtime diagnostics record created, reused, retained-entry, stale-evicted, budget-evicted, retained-byte, and budget-byte texture/buffer rows under `render.graph.execution.transient_pool.*`, so frame captures and automated diagnostics can distinguish first-frame allocation churn, steady-state reuse, stale cleanup, and memory-pressure cleanup.

Dynamic-resolution scale changes are validated against the same pool contract. The regression builds small graph frames that request a half-resolution color target, a full-resolution color target, and the half-resolution target again. The first two frames create one WGPU texture each and leave exactly two descriptor buckets in the pool; the third frame must reuse the half-resolution bucket without creating another texture. This proves render-scale 0.5 to 1.0 to 0.5 switching does not cause unbounded transient pool growth.

## SSR Mip Aliases

The screen-space reflection coarse pyramid resources remain view aliases into their parent pyramid mip levels. Parent textures may now be direct logical backings or slot-backed textures. `owned_texture_mip_view(...)` resolves through the logical-to-physical backing map before creating the mip view, so SSR aliases continue to work without requiring a separate owned texture for the coarse logical resource.

## Custom Owned Texture Views

Plan 11 IBL bake now needs storage texture views that are not the default sampled/render-attachment view bound at materialization time. `RenderGraphExecutionResources::owned_texture_view_with_descriptor(...)` resolves a logical graph texture name through its physical owned backing, validates the requested `wgpu::TextureViewDescriptor`, and then creates a fresh `wgpu::TextureView`.

The validation is intentionally local to the resource table:

- explicit view format must match the materialized WGPU texture format,
- explicit view usage must be contained in the texture usage derived from the graph `TextureDesc`,
- requested view dimension must be compatible with the graph texture dimension,
- mip range must stay inside `TextureDesc.mip_levels`,
- array-layer range must stay inside `TextureDesc.depth`.

This keeps environment-specific planning in `environment/ibl_bake_wgpu_command_plan.rs` while allowing the generic graph resource table to expose a Cube texture backing as a mip-scoped `D2Array` storage view for PMREM and IEM compute passes. It does not create bind groups, compute pipelines, upload parameter buffers, allocate readback buffers, or submit command encoders.

Graph-backed IBL PMREM/IEM readback resolves each live texture `Write` through its compiled access ID.
The transient access table retains one full WGPU texture handle per compiler physical allocation in
addition to subresource views; a multi-mip PMREM readback is admitted only when all live writes map
to that same allocation. Graph-backed IBL irradiance SH9 output and readback follow the same
exact-resource rule for buffers:
`ibl_bake_wgpu_dispatch.rs` resolves the declared output through
`RenderPassGpuExecutionContext::require_buffer_binding(...)`, preserving the compiled transient
offset and non-zero size in `StorageBufferRange`. The graph readback descriptor carries that same
range into staging `copy_buffer_to_buffer` and product diagnostic admission, so a non-zero transient
window is not read from offset zero. Direct environment-capture targets continue to use the legacy
full-buffer `StorageBuffer` variant because they are outside a compiled graph and are owned by the
capture target rather than the frame resource table.

Graph-owned persistent textures now have a separate frame-scoped exact lease table. Materialization
indexes each live persistent texture access by `RenderGraphResourceAccessId`, retains one WGPU texture
handle per resolved persistent backing resource, and materializes the compiler-projected subresource
view for each access. Equal `(backing, range)` scopes reuse one created WGPU view, while each access keeps
its own lease identity. `CompiledRenderGraph::persistent_texture_backing_resource(...)` normalizes a
logical texture-view alias to its persistent parent, so alias lifetime flags and logical names cannot
fork execution ownership. Sparse/provider-owned resources remain outside this table and fail closed at
the missing typed-lease boundary. `graph_owned_texture_view_for_access(...)` and
`graph_owned_texture_for_access(...)` select the transient physical-allocation table or this persistent
table without falling back to a resource name. Both tables are cleared before owned backings return to,
or retire into, the transient pool.

The standard resolver-backed texture view, descriptor, optional view, owned texture, physical texture,
full-mip, explicit-mip, and mip-count helper families now first resolve the compiled graph-owned access
ID. Normal view consumers receive the exact prebuilt view. Compatibility helpers that intentionally ask
for a full chain or a separately selected mip first validate the exact lease, then create/return that
explicit view contract. Buffer helpers remain on their independent exact byte-window path; this texture
cut does not widen a `wgpu::BufferBinding`.

Persistent exposure buffers use the imported-resource path rather than a second graph-owned
persistent-buffer table. The post-process descriptor publishes a 16-byte `STORAGE | COPY_SRC |
COPY_DST` schema for both exposure history slots. `exposure-resolve` declares compute storage reads
and read-write output, while `scene-composite`, `color-lut-bake`, and `uber` declare their actual
fragment/compute read stages. Pipeline authoring lowers those scopes through the versioned external
access APIs, and `CompiledRenderGraphExternalAccessPacket` retains one exact access-ID lease per live
consumer. The WGPU external binding table therefore resolves every exposure binding as `0..16`
instead of widening a report-only import by name. Split DoF, motion-blur, and blur passes remain
outside this dependency because their shaders do not sample resolved exposure.

Typed external texture accesses now materialize a physical view from the compiler packet rather
than cloning one producer default view for every access. When a provider publishes both the WGPU
texture backing and its physical descriptor, the frame table creates the exact mip/layer/aspect view
from `Texture(range)` and reuses one created view for equal `(graph resource, range)` scopes. Access
IDs remain distinct even when their views are shared. A provider that publishes only a view may reuse
that view only for a compiler-canonical range that covers the complete physical descriptor; a partial
mip/layer/plane request fails before encoding. Legacy `UnresolvedExternal` access remains an explicit
whole-view compatibility path rather than being reported as an exact lease.

TAA scene-color history is the first fixed-size temporal product family wired through that exact
external-texture path. Both ping-pong slots publish a View-sized `Rgba16Float`, one-mip descriptor
with `SAMPLED | RENDER_ATTACHMENT` usage. The previous slot declares a full-texture fragment sampled
read; the current slot declares a full-texture color-attachment write. `TemporalHistoryStore` retains
the texture/view lifetime pair, and the compiled-scene binder borrows the backing texture, default
view, physical descriptor, and stable identity into one external binding. The materializer can
therefore create the access-scoped view directly from the compiled packet. This fixed View schema is
not a template for HZB or volumetric history, whose mip/depth policies require a dynamic resource
catalog, nor for AO's explicit 1x1 fallback variant.

Hybrid GI and SSR now use the same fixed View-sized external-texture contract without sharing an
allocation owner. Hybrid GI's lighting and temporal-metadata previous slots are one-mip
`Rgba16Float` fragment inputs published by the plugin descriptor and backed by renderer history
textures. SSR's existing reprojection shader now receives its previous history through a declared
fragment sampled access instead of reading `SceneFrameHistoryTextures` directly from the GPU
executor. Missing cold-start bindings remain optional and select the existing fallback view while
the domain-validity flag disables blending. These changes make the compiled access packet the
execution authority. Uber keeps its current-Hybrid-GI-output-first policy, but its previous-GI
fallback is now another resolver-backed exact access; SSR auxiliary shader entries bind fallback
views instead of undeclared history-owner views. They do not change GI/SSR filtering weights or
claim a measured optimization.

`CompiledRenderPipeline` also freezes a `CompiledHistoryEpiloguePlan` from the final live writer access
of the canonical GI, SSR, HZB, and volumetric outputs. The frame-owned serial encoder still performs the
cross-frame copies so submission and completion ordering do not move prematurely, but history encoding
now resolves graph sources only through the compiled access IDs and fails before submission when the
physical lease is missing. A history output must declare `COPY_SRC` during pipeline compilation. This is
the source-side prerequisite for a future UE-style graph extraction/copy epilogue; barrier lowering,
queue ownership, dynamic WGPU validation, PNG, and RenderDoc evidence remain open.

## Runtime Prepare Collector Scene Resource Boundary

The 2026-07-07 editor command-palette validation exposed a crate-level compile blocker in `runtime_prepare_collector.rs`: production code needed `ResourceStreamer` and `MaterialCaptureSeed` while `graphics::scene::resources` was private to the scene module. `graphics::scene::mod.rs` now keeps `resources` visible only inside `crate::graphics`, and `runtime_prepare_collector.rs` imports those internal types through `graphics::scene::resources::{...}`. The 2026-07-08 closeout removed the stale test-only gates from `MaterialCaptureSeed`, `MaterialRuntime::capture_seed()`, and the material-capture accessor child because runtime prepare collectors now consume that neutral material/texture-sampling context in production. This is a boundary repair only; it does not make the `resources` folder public, does not change graph resource materialization, and does not add a renderer facade.

## Validation State

The materialization and pool source tests cover compatible non-overlapping textures sharing one bucketed owned WGPU backing, descriptor-incompatible textures arriving in separate graph buckets and distinct bucketed physical labels, compatible non-overlapping buffers sharing one bucketed WGPU backing, transient pool reuse across frames, stale pool entry eviction, budget pressure evicting retained entries down to the configured byte cap, render-scale 0.5 to 1.0 to 0.5 switching retaining bounded descriptor buckets, and stale logical texture/buffer bindings being rejected when they are not part of the compiled live lifetime set. The runtime diagnostics contract also asserts the `render.graph.execution.transient_pool.*` count/byte series and `render.graph.materialization.stale_*` count rows when the lib-test crate can compile.

The 2026-08-30 persistent exact-view source slice adds direct-mip materialization coverage and a
persistent-parent texture-view-alias resolver regression. The alias regression asserts the compiled
parent backing identity, materializes the per-access WGPU view lease, and resolves it through the
pass-scoped resolver. Exact rustfmt, locked Cargo metadata, scoped diff checking, and source-contract
classification passed. Managed Cargo/WGPU execution, PNG/RDC capture, timing, VRAM, power, and
coordinator acceptance remain pending; no dynamic rendering or performance result is claimed.

The 2026-08-30 exposure external-buffer continuation adds builder, feature-descriptor, compiled
pipeline, and external-packet contract tests for the exact 16-byte lease and its producer provenance.
Scoped rustfmt, locked Cargo metadata, source-contract checks, and scoped diff checking passed. The
managed Cargo/WGPU lane is still blocked before test execution, so framebuffer PNG, RenderDoc RDC,
GPU timing, VRAM, power, and visual acceptance remain pending.

The 2026-08-30 provider-owned external-texture continuation adds source regressions for an exact mip2
view created from a physical backing, rejection of a partial view-only lease, and compatibility for a
canonical full-scope view-only lease. The materializer caches equal physical scopes instead of creating
one WGPU view per access. Managed Cargo/WGPU execution, framebuffer PNG, RenderDoc RDC, timing, VRAM,
power, and coordinator acceptance remain pending; no dynamic rendering or performance result is
claimed.

The 2026-08-30 TAA continuation adds source regressions across feature authoring, compiled external
packet retention, history backing publication, and compiled-scene binding. Exact rustfmt, locked Cargo
metadata, scoped source-contract checks, and scoped diff checking passed. Managed Cargo remains
blocked before compilation by `cargo_reuse_target_mismatch`; no WGPU execution, consecutive-frame
PNG, RenderDoc RDC, timing, VRAM, power, or visual acceptance is claimed.

The 2026-08-30 Hybrid GI/SSR continuation adds plugin descriptor, compiled SSR packet,
resolver-only SSR execution, and physical history binder source regressions. Exact rustfmt, locked
metadata, source-contract, and scoped diff checks passed. Managed Cargo/WGPU, consecutive-frame PNG,
RenderDoc, timing, VRAM, power, and visual acceptance remain pending.

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed on 2026-06-12 after the pool diagnostics bridge with existing warnings only. Earlier focused lib-test commands were blocked before running their filtered tests by unrelated `zircon_runtime` lib-test compile errors in `zircon_runtime/src/ui/tests/runtime_input_manager.rs` and `zircon_runtime/src/ui/tests/style_mapping.rs`; an earlier materialization test attempt was also blocked by the dirty `zircon_runtime/src/scene/tests/ecs_schedule.rs` test source. The stale-binding follow-up later compiled the focused lib-test binary with `cargo test -p zircon_runtime --lib materialization_validation_rejects_stale --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-stale-lifetime-validation-0618 --message-format short --color never` in 8m06s, then direct exact execution of `materialization_validation_rejects_stale_texture_binding_outside_live_lifetimes` and `materialization_validation_rejects_stale_buffer_binding_outside_live_lifetimes` both passed.

On 2026-07-06, `materialization_exposes_owned_cube_storage_texture_array_views` passed through a real offscreen WGPU backend. The test materializes a `Rgba16Float` Cube transient texture with storage usage, creates mip2 as a `D2Array` storage texture view with six layers, and verifies that invalid mip ranges, invalid array ranges, and illegal view usages fail before `Texture::create_view(...)`.

The 2026-06-17 RG-M2 budget follow-up passed `rustfmt --edition 2021` over the touched Rust files and `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-transient-pool-budget-0617` twice: first in 573.2 seconds with the existing warning set, then again in 44.3 seconds after removing an unrelated unused compiled-cache re-export warning from the touched render pipeline module. `git diff --check -- <RG-M2 scoped files>` passed with only Git LF-to-CRLF notices. Focused pool and runtime-diagnostics tests remain deferred for the implementation-first phase.

The 2026-06-22 F16 compiled-scene structure split passed scoped `rustfmt --edition 2021 --check`, scoped `git diff --check` with only LF-to-CRLF notices for the first resource/stage slice, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-f16-0622-coremin --message-format short --color never` after the resource/stage split, after the present/readback/pool-release owner split, and after the later F3 `Arc` handoff plus sprite-stage fixture fix, with the existing 143-warning set. The focused `active_late_graph_stages_follow_compiled_pipeline_order` lib-test passed 1/1, the focused `active_sprite_graph_stages_follow_unculled_sprite_passes` lib-test passed 1/1 after the synthetic graph passes were marked as side-effect roots for the fixture, and the broader `compiled_scene` lib-test filter passed 22/22 with the existing warning set.

The 2026-06-17 RG-M4 alias/profile follow-up passed `rustfmt --edition 2021` over the touched Rust files. The first scoped `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-alias-profile-0617` timed out after 124 seconds during cold-target compilation, then the warmed rerun completed in 301.3 seconds with the existing warning set. The runtime-diagnostics fixture and assertions now cover alias-count rows plus CPU profile count/total/max microsecond rows, but focused tests remain deferred for the implementation-first phase.

The 2026-06-17 graph materialization validation follow-up passed `rustfmt --edition 2021` over the touched Rust files. The first scoped `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-materialization-0617` timed out after 244 seconds during cold-target compilation, then the warmed rerun completed in 188 seconds with the existing warning set. `git diff --check -- <materialization scoped files>` passed with only Git LF-to-CRLF notices, and the conflict-marker scan found no hits. Focused materialization/runtime-diagnostics tests remain deferred for the implementation-first phase.

The 2026-06-17 HZB executor-owned external binding follow-up passed `rustfmt --edition 2021` over the touched Rust files. The first scoped `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-materialization-0617` timed out after 364 seconds during cold-target compilation without a Rust diagnostic, then the warmed rerun completed in 73.5 seconds with the existing warning set. The source-contract test `hzb_external_fallback_buffers_satisfy_materialization_report` was authored but not run per the implementation-first direction.

The 2026-06-17 required External binding contract follow-up passed `rustfmt --edition 2021` over the touched Rust files and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-binding-contract-0617 --message-format short --color never` with the existing warning set. It adds the source-contract test `materialization_validation_fails_unbound_required_external_buffer` and extends HZB descriptor/pipeline compile tests to assert required-buffer metadata, but those focused tests remain deferred for the implementation-first phase.

The 2026-06-17 required External texture and modularization follow-up moved materialization validation into `graph_execution/materialization_validation.rs`, moved render-pipeline external resource merge planning into `render_pipeline_asset/graph_resources.rs`, added required external texture descriptor helpers, and added source-contract tests for required texture lifetimes plus external texture/buffer binding conflicts. The same path now covers production `SHADOW_ATLAS`: builtin shadow/mesh/deferred descriptors declare it as required texture, the render path imports the persistent atlas view through graph-lifetime-aware frame-resource binding, and the dedicated shadow atlas compile tests assert default Forward+ and Deferred graph lifetimes/read-write rows. `rustfmt --edition 2021` passed over the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-required-external-texture-0617 --message-format short --color never` passed with the existing warning set. Focused tests remain authored but not executed in this implementation-first slice.

The 2026-06-17 typed optional External ownership follow-up adds report-only typed texture/buffer descriptor helpers and preserves their bindings through pipeline compile and materialization validation. Built-in optional texture/buffer externals and first-party particles/Hybrid GI/Virtual Geometry plugin descriptors now declare their external type instead of using unknown report-only resources. `materialization_validation_reports_unbound_typed_optional_external_without_failing` covers the execution-side report semantics; `typed_optional_external_tests.rs` covers compile-time preservation and optional texture/buffer conflict rejection. `rustfmt --edition 2021` and `rustfmt --edition 2021 --check` passed over the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-typed-optional-external-0617 --message-format short --color never` passed with the existing warning set. A package-scoped plugin check for particles, Hybrid GI, and Virtual Geometry was blocked before compilation because `zircon_plugins/Cargo.lock` would need an update under `--locked`; the lockfile was not changed.

The 2026-06-17 `RgResourceResolver` cutover follow-up renamed the pass resource resolver, added pass-declared access checks in front of physical texture/buffer lookup helpers, and moved post-process required resource validation plus terminal input selection onto resolver-backed name access. `resolver_backed_name_access_ignores_stale_context_resource_rows` covers stale copied context rows, while `rg_resource_resolver_requires_pass_declared_access_before_physical_texture_lookup` covers physical lookup rejecting undeclared access before reaching the WGPU table. `rustfmt --edition 2021` passed over the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-resolver-cutover-0617 --message-format short --color never` passed with the existing 74-warning set.

The 2026-06-23 Plan 07 built-in post-process executor owner split keeps `graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs` as the registry-facing executor surface while moving helper ownership into child modules. `builtin_postprocess_executors/frame_effects.rs` owns frame effect predicates such as TAA, reconstructed motion vectors, depth of field, and screen-space reflection usage. `builtin_postprocess_executors/graph_resources.rs` owns `product_postprocess_executor(...)`, pass resource kind lookup, and external texture/buffer validation before executor dispatch. `builtin_postprocess_executors/resource_routing.rs` owns output-transfer, bloom, and uber input/output resource selection plus the former inline routing tests. The structure guard `runtime_15_builtin_postprocess_executors_are_folder_backed` and status anchor `render_plan07_builtin_postprocess_executor_owner_split_static_passed` lock this split, including the paths `graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs`, `graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs`, and `graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs`. Scoped rustfmt/static/line-count/docs-anchor/diff-check and locked core-min `cargo check` passed with existing warnings; the focused locked structure Cargo test is blocked before compilation by the current `Cargo.lock` update requirement.

The 2026-06-23 Plan 01 render graph execution record owner split keeps `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs` as the execution-record aggregation surface while moving compute workload and tests into child modules. `render_graph_execution_record/compute_workload.rs` owns `RenderGraphComputeDispatchRecord`, `RenderGraphComputeWorkloadDispatchContext`, dispatch group sizing, `RenderGraphComputeWorkloadAuditRecord`, audit status, and the compute audit tests. `render_graph_execution_record/tests.rs` owns non-compute record behavior tests for resource reports, light-grid reports, queue/stage metadata, dependencies, and debug markers. The structure guard `runtime_15_render_graph_execution_record_is_folder_backed` and status anchor `render_plan01_execution_record_owner_split_static_passed` lock this split, including the paths `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs` and `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs`. Scoped rustfmt/static/line-count/docs-anchor/diff-check passed; locked core-min Cargo check was blocked before compilation by the current `Cargo.lock` update requirement. No new Cargo/WGPU/RenderDoc pass is claimed.

The 2026-06-24 Plan 01 Render pass executor registry test owner split keeps `graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs` as a root test module for shared imports, support fixtures, plugin policy tests, and child owner mounts. `render_pass_executor_registry/tests/registry_contracts.rs` owns registry/compiled-pipeline contract tests, `render_pass_executor_registry/tests/postprocess_context_guards.rs` owns temporal/postprocess/SSR missing-context guards and effect-stack fixtures, and `render_pass_executor_registry/tests/renderer_context_guards.rs` owns UI/overlay/sprite/mesh/prepass/shadow/deferred WGPU context guards plus the shadow-atlas texture fixture. The structure guard `runtime_15_render_pass_executor_registry_tests_are_child_owners` and status anchor `render_plan01_executor_registry_test_owner_split_static_passed_cargo_deferred_active_compile_lane` lock this split without changing WGPU executor behavior. Scoped rustfmt/static/line-count/docs-anchor/whitespace/diff-check passed; Cargo/WGPU/RenderDoc remains deferred while active compile lanes are present.

The 2026-07-14 Runtime15 follow-up preserves the Plan 01 command-list split and moves the remaining mesh pass-recording responsibility from `render_pass_execution_context/gpu.rs` into `render_pass_execution_context/gpu/mesh_recording.rs`. The three current owners are 387/146/516 lines after rustfmt. The child retains the original function signatures, attachment operations, command replay order, advanced PBR/transmission routing, error messages and resource access modes; no compatibility shim or duplicate path remains. Guard `runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner` locks mounts, moved anchors, documentation and the three-file 800-line budget.

The follow-up GPU context propagation slice passes the resolver from `RenderPassExecutionContext` into `RenderPassGpuExecutionContext` and migrates deferred lighting texture/buffer lookup onto resolver-backed helpers in `gpu/resource_lookup.rs`. The next deferred scene bridge slice extracts `gpu/deferred.rs` and moves depth-prepass plus deferred G-buffer lookups onto the same compiled-pass access gate; `gpu.rs` is 810 lines after the split. The mesh bridge slice moves mesh-stage attachments, optional light-grid buffers, and TAA reactive-mask mesh attachments onto resolver-backed helpers; `gpu.rs` is 839 lines and `gpu/resource_lookup.rs` is 70 lines after that slice. The scene surface bridge slice extracts `gpu/surface.rs` for sprite, screen-space UI, preview-sky, and overlay bridge lookup; `gpu.rs` is 730 lines, `gpu/surface.rs` is 158 lines, and `gpu/resource_lookup.rs` is 70 lines after that split. The particle bridge slice moves `scene-color` write / `scene-depth` read transparent rendering and `scene-velocity` write / `scene-depth` read velocity rendering through resolver-backed helpers in `gpu/particle.rs`, removes the particles plugin executor's redundant direct `gpu.resources.require_texture_view(...)` precheck, and leaves `gpu/particle.rs` at 76 lines. The SSR bridge slice moves resolve, reflection-pyramid, coarse-pyramid, and specular-occlusion graph inputs/outputs plus mip-target alias lookup through resolver-backed helpers in `gpu/post_process/screen_space_reflection.rs`; the optional shared-input helpers preserve fallback textures when a pass does not declare a slot, and the SSR descriptor now declares `light-list` for the shared post-process bind group. The root postprocess bridge slice moves stack/color-LUT/effect-chain/compute-resource/temporal/terminal lookups through the same helpers, adds `gpu/post_process/{effects,computed_resources,temporal,terminal}.rs`, leaves root `gpu/post_process.rs` at 490 lines, and keeps SSR mip-count metadata behind a resolver-gated helper. `rustfmt --edition 2021` and the follow-up `--check` passed over the touched Rust files. The latest `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-resolver-cutover-0617 --message-format short --color never` rerun passed with the existing 142-warning set. The package-scoped particles plugin check still stops before compilation because `zircon_plugins/Cargo.lock` would need an update under `--locked`. Remaining direct GPU lookup work is now `RenderGraphExecutionResources` visibility tightening plus the existing `gpu/hzb_occlusion.rs` legacy lookup; helper fallback/internal calls remain by design.

## Device-qualified executor caches

`RenderGraphExecutionResources` records the device identity that materialized the current graph, and
`RenderPassGpuExecutionContext::device_epoch()` exposes that fact as an opaque, comparable
`RenderPassDeviceEpoch`. Persistent native objects owned by an executor must not infer compatibility
from executor registration lifetime or from texture formats alone.

Core advanced-lighting executors use the private `RenderPassDeviceEpochCache<K, V>`. Its identity is
the materialized device epoch plus the local pipeline descriptor key. Stable frames reuse the cached
value; an epoch or key change drops the complete old entry before constructing and publishing its
replacement. Failed replacement leaves the cache empty, so an old-generation pipeline can never be
used as fallback. Froxel, OIT, planar filtering, and the shared SSS bundle require the epoch before
resource lookup or GPU command encoding; in particular, OIT counter clear occurs only after current-
epoch pipeline admission.

The stable-frame source upper bound is nine constant-time epoch/key comparisons when all affected
passes execute. This is not measured timing. The 2026-08-31 slice has source contracts, drop-order unit
tests, rustfmt, scoped diff checking, and locked metadata only; managed Cargo/WGPU, live device-loss
recovery, fresh PNG/RDC, RenderDoc markers, 300-frame timing, VRAM, and power evidence remain pending.

Transient texture and buffer allocations use the same `RenderPassDeviceEpoch` owner as graph
execution resources and persistent executor caches. The pool converts the current RHI profile into
that opaque value once per frame; submission-ticket scalar access is confined to the ticket-admission
boundary. This keeps free and pending allocation retirement on one graph epoch identity without
changing descriptor-key reuse, completion qualification, or budget-eviction ordering.
