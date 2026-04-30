---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/core_profiles.rs
  - zircon_runtime/src/plugin/package_manifest/mod.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_module_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_module_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/package_manifest/builtin_catalog.rs
  - zircon_runtime/src/plugin/component_type_descriptor/mod.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_type_descriptor.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_property_descriptor.rs
  - zircon_runtime/src/plugin/component_type_descriptor/constructors.rs
  - zircon_runtime/src/plugin/ui_component_descriptor.rs
  - zircon_runtime/src/plugin/extension_registry/mod.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_module.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_ui.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/ui/component/catalog.rs
  - zircon_runtime/src/ui/component/descriptor.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/mod.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection_builder.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection_access.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_state.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/default_packaging.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/default_true.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/export_generated_file.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/generated_files.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/export_build_plan/asset_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize.rs
  - zircon_runtime/tests/export_build_plan_contract.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_loader.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_candidate.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/collect_manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/dynamic_library_name.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mod.rs
  - zircon_runtime/src/plugin/runtime_plugin/mod.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/project_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/tests/native_plugin_loader_contract.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/read.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/create_default_pipelines.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/scene_renderer_advanced_plugin_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_cull_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_render_path_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_indirect_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_cull_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_render_path_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_indirect_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_readback_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/virtual_geometry_output_updates/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/virtual_geometry_output_updates/cull_output_update.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/virtual_geometry_output_updates/render_path_output_update.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/virtual_geometry_output_updates/indirect_output_update.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/virtual_geometry_output_updates/last_output_update.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass/output.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass/store_parts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass/records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass/buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass/output.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass/store_parts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass/entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass/buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_node_and_cluster_cull_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_node_and_cluster_cull_pass/output.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_node_and_cluster_cull_pass/store_parts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_node_and_cluster_cull_pass/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_node_and_cluster_cull_pass/page_requests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/output.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/selection_collection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/selection_filter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/ordering.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/frontier_ranking.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/build_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/build_selections.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass/seed_backed_execution_selection/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats/execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats/execution_owned_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_prepare_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_source.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_options.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/default.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/methods.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/pipeline/validation/validate_renderer_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compiled_feature_names/compiled_feature_names.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_history.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_surface_cache.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/graphics/tests/m5_flagship_slots.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_args_source_authority.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_args_authority.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_execution_stats.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_node_and_cluster_cull_execution.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_prepare_render.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_unified_indirect.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_authority.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_event/runtime/editor_event_runtime_inner.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/project.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/capabilities.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/selection_policy.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/cargo_invocation.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/cargo_build.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/generated_files.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/diagnostics.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/registration_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/module_crate_lookup.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/module_capabilities.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/project_selection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/native_project_selection.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_enable_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_selection_update_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_load_state.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/mod.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/native_dynamic_preparation.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cleanup.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cargo_build.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/staging.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/artifacts.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/package_metadata.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs
  - zircon_editor/ui/workbench/module_plugins_pane.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/host/builtin_layout/ensure_shell_instances.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_registry.rs
  - zircon_editor/src/ui/workbench/view/view_registry_register_view.rs
  - zircon_editor/src/ui/workbench/view/view_registry_open_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_registry_restore_instance.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/README.md
  - zircon_plugins/editor_support/Cargo.toml
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/service_types.rs
  - zircon_plugins/physics/editor/Cargo.toml
  - zircon_plugins/physics/editor/src/lib.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/editor/Cargo.toml
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_plugins/texture/editor/Cargo.toml
  - zircon_plugins/texture/editor/src/lib.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/lib.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/editor/Cargo.toml
  - zircon_plugins/navigation/editor/src/lib.rs
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/editor/Cargo.toml
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/service_types.rs
  - zircon_plugins/animation/runtime/src/sequence_runtime.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/lib.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/editor/Cargo.toml
  - zircon_plugins/virtual_geometry/editor/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/editor/Cargo.toml
  - zircon_plugins/hybrid_gi/editor/src/lib.rs
  - zircon_plugins/runtime_diagnostics/plugin.toml
  - zircon_plugins/runtime_diagnostics/editor/Cargo.toml
  - zircon_plugins/runtime_diagnostics/editor/src/lib.rs
  - zircon_plugins/ui_asset_authoring/plugin.toml
  - zircon_plugins/ui_asset_authoring/editor/Cargo.toml
  - zircon_plugins/ui_asset_authoring/editor/src/lib.rs
  - zircon_plugins/native_window_hosting/plugin.toml
  - zircon_plugins/native_window_hosting/editor/Cargo.toml
  - zircon_plugins/native_window_hosting/editor/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
implementation_files:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/core_profiles.rs
  - zircon_runtime/src/plugin/package_manifest/mod.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_module_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_module_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/package_manifest/builtin_catalog.rs
  - zircon_runtime/src/plugin/component_type_descriptor/mod.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_type_descriptor.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_property_descriptor.rs
  - zircon_runtime/src/plugin/component_type_descriptor/constructors.rs
  - zircon_runtime/src/plugin/ui_component_descriptor.rs
  - zircon_runtime/src/plugin/extension_registry/mod.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_module.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/mod.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection_builder.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection_access.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_state.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/default_packaging.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/default_true.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/export_generated_file.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/generated_files.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/export_build_plan/asset_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_loader.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_candidate.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/collect_manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/dynamic_library_name.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mod.rs
  - zircon_runtime/src/plugin/runtime_plugin/mod.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/project_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/read.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/create_default_pipelines.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_source.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_options.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/default.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/methods.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/pipeline/validation/validate_renderer_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compiled_feature_names/compiled_feature_names.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_event/runtime/editor_event_runtime_inner.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/project.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/capabilities.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/cargo_invocation.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/cargo_build.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/generated_files.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/diagnostics.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/registration_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/module_crate_lookup.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/module_capabilities.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/project_selection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/native_project_selection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_enable_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_load_state.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/mod.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/native_dynamic_preparation.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cleanup.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cargo_build.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/staging.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/artifacts.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/package_metadata.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs
  - zircon_editor/ui/workbench/module_plugins_pane.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/host/builtin_layout/ensure_shell_instances.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_registry.rs
  - zircon_editor/src/ui/workbench/view/view_registry_register_view.rs
  - zircon_editor/src/ui/workbench/view/view_registry_open_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_registry_restore_instance.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/README.md
  - zircon_plugins/editor_support/Cargo.toml
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/service_types.rs
  - zircon_plugins/physics/editor/Cargo.toml
  - zircon_plugins/physics/editor/src/lib.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/editor/Cargo.toml
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_plugins/texture/editor/Cargo.toml
  - zircon_plugins/texture/editor/src/lib.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/lib.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/editor/Cargo.toml
  - zircon_plugins/navigation/editor/src/lib.rs
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/editor/Cargo.toml
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/service_types.rs
  - zircon_plugins/animation/runtime/src/sequence_runtime.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/lib.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/editor/Cargo.toml
  - zircon_plugins/virtual_geometry/editor/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/editor/Cargo.toml
  - zircon_plugins/hybrid_gi/editor/src/lib.rs
  - zircon_plugins/runtime_diagnostics/plugin.toml
  - zircon_plugins/runtime_diagnostics/editor/Cargo.toml
  - zircon_plugins/runtime_diagnostics/editor/src/lib.rs
  - zircon_plugins/ui_asset_authoring/plugin.toml
  - zircon_plugins/ui_asset_authoring/editor/Cargo.toml
  - zircon_plugins/ui_asset_authoring/editor/src/lib.rs
  - zircon_plugins/native_window_hosting/plugin.toml
  - zircon_plugins/native_window_hosting/editor/Cargo.toml
  - zircon_plugins/native_window_hosting/editor/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
plan_sources:
  - user: 2026-04-27 Native Dynamic 真库样例闭环
  - user: 2026-04-27 zircon_plugins 全量插件化收敛规划
  - user: 2026-04-28 继续完成 runtime plugin ComponentTypeRegistry gate
  - user: 2026-04-28 继续接通 RuntimeExtensionRegistry 到 World ComponentTypeRegistry
  - user: 2026-04-28 继续接通 runtime plugin UI component 到 runtime UI component catalog
  - user: 2026-04-28 继续收束 runtime plugin manager contribution duplicate diagnostics
  - user: 2026-04-28 继续收紧动态插件组件属性写入的 ComponentTypeDescriptor editable 契约
  - user: 2026-04-28 继续收束 NativeDynamic 导出目录去重
  - user: 2026-04-28 继续收束 native full diagnostic loader 的 split runtime/editor 动态库状态
  - user: 2026-04-28 继续硬切 native loader full/target-specific API 命名与插件窗口状态聚合
  - user: 2026-04-30 实现 Native ABI v2 最小生产切片
  - user: 2026-04-30 继续 Native ABI v2 M0 行为边界
  - docs/superpowers/specs/2026-04-27-native-dynamic-fixture-closure-design.md
  - docs/superpowers/plans/2026-04-27-native-dynamic-fixture-closure.md
  - docs/superpowers/plans/2026-04-29-runtime-dynamic-pluginized-runtime-aggressive-migration.md
  - user: 2026-04-26 Runtime/Editor 最小本体与发行导出插件化设计
  - .codex/plans/Runtime_Editor 最小本体与发行导出插件化设计.md
  - .codex/plans/全系统重构方案.md
tests:
  - Windows Task 12: .\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4 (passed)
  - Windows Task 12: .\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4 (passed)
  - Windows Task 12: git diff --check -- .codex .opencode docs zircon_runtime zircon_plugins zircon_editor zircon_app (exit 0; LF-to-CRLF warnings only)
  - Windows Task 12: cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (1 passed)
  - Windows Task 12: cargo test -p zircon_runtime --test export_build_plan_contract --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (8 passed)
  - Windows Task 12: cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" (passed)
  - Windows Task 12: cargo test -p zircon_runtime --lib export_build_plan --no-default-features --features core-min --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (11 passed)
  - Windows Task 12: cargo test -p zircon_runtime --lib runtime_modules --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (5 passed)
  - Windows Task 12: cargo test -p zircon_runtime --lib plugin_render_feature --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (7 passed)
  - Windows Task 12: cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" (passed)
  - Windows Native ABI v2 M0: cargo test -p zircon_runtime --test native_plugin_loader_contract native_loader_exposes_v2_behavior_boundary_from_real_fixture --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never -- --nocapture (1 passed)
  - Windows Native ABI v2 M0: cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never -- --nocapture (2 passed)
  - Windows Native ABI v2 M0: rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/lib.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/tests/native_plugin_loader_contract.rs zircon_plugins/native_dynamic_fixture/native/src/lib.rs (passed)
  - Windows Native ABI v2 M0: cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never (passed)
  - Windows Native ABI v2 M0: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never (passed with existing render graph dead-code warning)
  - Windows Native ABI v2 M0: cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0-core" --color never (passed with existing render graph dead-code warning)
  - Windows Native ABI v2 M0: git diff --check -- zircon_runtime/src/plugin/native_plugin_loader zircon_runtime/src/lib.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/tests/native_plugin_loader_contract.rs zircon_plugins/native_dynamic_fixture/native/src/lib.rs docs/engine-architecture/runtime-editor-pluginized-export.md .codex/sessions/20260426-2305-render-feature-plugin-cutover.md (exit 0; LF-to-CRLF warnings only)
  - Windows Native ABI v2 M0 review fix: cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never -- --nocapture (2 passed)
  - Windows Native ABI v2 M0 review fix: cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never (passed)
  - Windows Native ABI v2 M0 review fix: rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/lib.rs zircon_runtime/src/plugin/mod.rs zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/tests/native_plugin_loader_contract.rs zircon_plugins/native_dynamic_fixture/native/src/lib.rs (passed)
  - Windows Native ABI v2 M0 review fix: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never (passed with existing render graph dead-code warning)
  - Windows Native ABI v2 M0 review fix: cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0-core" --color never (passed with existing render graph dead-code warning)
  - Windows Native ABI v2 M0 review fix: git diff --check -- zircon_runtime/src/plugin/native_plugin_loader zircon_runtime/src/plugin/mod.rs zircon_runtime/src/lib.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/tests/native_plugin_loader_contract.rs zircon_plugins/native_dynamic_fixture/native/src/lib.rs docs/engine-architecture/runtime-editor-pluginized-export.md .codex/sessions/20260426-2305-render-feature-plugin-cutover.md (exit 0; LF-to-CRLF warnings only)
  - blocked: cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-native-abi-v2-m0" --color never -- --nocapture (blocked before native-loader test by unrelated lib-test compile errors in asset VG cook exports and Hybrid GI split-light fixtures)
  - Windows Task 12: cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_sound_runtime -p zircon_plugin_net_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (76 runtime plugin tests plus doctests passed)
  - Windows Task 12: cargo test -p zircon_editor --lib editor_runtime_ --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (12 passed)
  - Windows Task 12: cargo test -p zircon_editor --lib native_aware_completion_aggregates_native_module_target_modes --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (1 passed)
  - Windows Task 12: cargo test -p zircon_editor --lib native_selection_aggregates_runtime_and_editor_module_target_modes --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture (1 passed)
  - Windows Task 12: cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_editor_support -p zircon_plugin_physics_editor -p zircon_plugin_animation_editor -p zircon_plugin_sound_editor -p zircon_plugin_net_editor -p zircon_plugin_navigation_editor -p zircon_plugin_particles_editor -p zircon_plugin_texture_editor -p zircon_plugin_virtual_geometry_editor -p zircon_plugin_hybrid_gi_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor -p zircon_plugin_native_window_hosting_editor --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" (passed)
  - Windows Task 12: .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir "D:\cargo-targets\zircon-runtime-dynamic-milestone" (passed cargo build --workspace --locked and cargo test --workspace --locked)
  - not run: 2026-04-28 editor manifest completion module split per user instruction to skip validation
  - not run: 2026-04-28 editor plugin enablement module split per user instruction to skip validation
  - not run: 2026-04-28 editor plugin status module split per user instruction to skip validation
  - not run: 2026-04-28 editor export build module split per user instruction to skip validation
  - not run: 2026-04-28 native dynamic export preparation split per user instruction to skip validation
  - not run: 2026-04-28 particle render feature moved from runtime core descriptor to particles plugin per user instruction to avoid frequent validation
  - not run: 2026-04-28 default forward/deferred pipelines stopped embedding particles and runtime particle rendering now requires the particle plugin descriptor; validation deferred per aggressive migration instruction
  - not run: 2026-04-28 RenderPipelineCompileOptions gained plugin-feature-name disable gates so particle quality profiles still disable plugin-owned particle rendering; validation deferred per aggressive migration instruction
  - not run: 2026-04-28 base render pass executor registry stopped registering particle.transparent by default; linked particle descriptors now own the executor-id admission path
  - not run: 2026-04-28 Virtual Geometry seed-backed execution selection hard-cut renamed former transition-path symbols to baseline/fixed-fanout names; validation deferred per aggressive migration instruction
  - WSL red/green: cargo test -p zircon_runtime --test export_build_plan_contract native_dynamic_strategy_only_loads_native_packaged_selections --no-default-features --features core-min --locked --jobs 1 -- --nocapture (first failed with native_dynamic_packages ["physics", "virtual_geometry"], then passed with only ["virtual_geometry"])
  - WSL red/green: cargo test -p zircon_runtime --test export_build_plan_contract source_template_without_library_embed_serializes_selection_without_linking_crate --no-default-features --features core-min --locked --jobs 1 -- --nocapture (first failed because SourceTemplate-only still populated linked_runtime_crates, then passed with serialized selection data but no plugin crate dependency or registration call)
  - WSL red/green: cargo test -p zircon_runtime --test export_build_plan_contract native_dynamic_selection_requires_native_dynamic_profile_strategy --no-default-features --features core-min --locked --jobs 1 -- --nocapture (first failed because a NativeDynamic selection was copied without profile strategy support, then passed with a planner diagnostic and no native loader manifest)
  - WSL red/green: cargo test -p zircon_runtime --test export_build_plan_contract library_embed_selection_without_source_or_library_profile_reports_unexported_plugin --no-default-features --features core-min --locked --jobs 1 -- --nocapture (first failed because a LibraryEmbed selection with a NativeDynamic-only profile produced no artifact and no diagnostic, then passed with a planner diagnostic)
  - WSL: cargo test -p zircon_runtime --test export_build_plan_contract --no-default-features --features core-min --locked --jobs 1 -- --nocapture (4 passed)
  - WSL: cargo test -p zircon_runtime --lib export_build_plan --no-default-features --features core-min --locked --jobs 1 -- --nocapture (10 passed)
  - WSL: cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 (passed with CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-export-matrix-wsl)
  - WSL: cargo test -p zircon_runtime --test export_build_plan_contract --locked --jobs 1 -- --nocapture (4 passed)
  - WSL: cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 -- --nocapture (30 passed)
  - WSL red/green: cargo test -p zircon_runtime --test export_build_plan_contract library_embed_deduplicates_runtime_crate_dependencies_and_registration_calls --no-default-features --features core-min --locked --jobs 1 -- --nocapture (first failed with duplicate linked runtime crates, then passed with one linked crate, one Cargo dependency, and one registration call)
  - WSL: cargo test -p zircon_runtime --test export_build_plan_contract --no-default-features --features core-min --locked --jobs 1 -- --nocapture (5 passed)
  - WSL: cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 (passed with CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-export-matrix-wsl)
  - WSL: cargo test -p zircon_runtime --test export_build_plan_contract --locked --jobs 1 -- --nocapture (5 passed)
  - WSL: cargo test -p zircon_runtime --lib export_build_plan --no-default-features --features core-min --locked --jobs 1 -- --nocapture (blocked before export tests by active render test unresolved import `VirtualGeometryNodeAndClusterCullPassStoreParts` in `virtual_geometry_executed_cluster_selection_pass/tests/mod.rs`)
  - WSL red/green: cargo test -p zircon_runtime --test export_build_plan_contract native_dynamic_deduplicates_loader_manifest_packages_by_plugin_id --no-default-features --features core-min --locked --jobs 1 -- --nocapture (first failed with duplicate native package ids, then passed with one native package and one loader manifest row)
  - TDD red: cargo test -p zircon_runtime --test export_build_plan_contract native_dynamic_deduplicates_loader_manifest_packages_by_output_directory --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-directory-dedupe -- --nocapture (failed because `native_dynamic_packages` contained both `physics.debug` and `physics_debug` even though both sanitize to `plugins/physics_debug`)
  - blocked green: cargo test -p zircon_runtime --test export_build_plan_contract native_dynamic_deduplicates_loader_manifest_packages_by_output_directory --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-directory-dedupe -- --nocapture (blocked before the export test by active-owner compile errors in `zircon_runtime/src/ui/component/descriptor.rs` duplicate `state`/`slot` methods and private VG pass accessors in the seed-backed execution-selection owner)
  - inconclusive green retry: cargo test -p zircon_runtime --test export_build_plan_contract native_dynamic_deduplicates_loader_manifest_packages_by_output_directory --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-directory-dedupe -- --nocapture (progressed to compiling `zircon_runtime`, emitted an unrelated VG unused-import warning, then timed out before a test summary while many active Cargo/rustc jobs were present)
  - not rerun: 2026-04-28 18:22 process scan still showed unrelated active Cargo/rustc jobs for editor build/menu/operation tests, hybrid-GI tests, and SRP/RHI checks, so no NativeDynamic output-directory GREEN claim was made
  - green: cargo test -p zircon_runtime --test export_build_plan_contract native_dynamic_deduplicates_loader_manifest_packages_by_output_directory --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-directory-dedupe -- --nocapture (1 passed, 0 failed)
  - green: cargo test -p zircon_runtime --test export_build_plan_contract --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-directory-dedupe -- --nocapture (8 passed, 0 failed)
  - green: cargo test -p zircon_runtime --test export_build_plan_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-validation-ladder -- --nocapture (8 passed, 0 failed; emitted unrelated default-feature `scene_renderer::ui::sdf_atlas` dead-code warnings)
  - green: cargo test -p zircon_runtime --lib export_build_plan --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-validation-ladder-core-min -- --nocapture (11 passed, 0 failed)
  - green: cargo test -p zircon_runtime --lib export_build_plan --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-validation-ladder -- --nocapture (11 passed, 0 failed; emitted unrelated default-feature VG/GI readback completion-part unused-import warnings)
  - WSL: cargo test -p zircon_runtime --test export_build_plan_contract --no-default-features --features core-min --locked --jobs 1 -- --nocapture (6 passed)
  - WSL: cargo test -p zircon_runtime --test export_build_plan_contract --locked --jobs 1 -- --nocapture (6 passed)
  - WSL: cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 (blocked outside export planning by active render private re-export errors for `VirtualGeometryGpuResources`)
  - WSL: cargo test -p zircon_runtime --test native_plugin_loader_contract --no-default-features --features core-min --locked --jobs 1 -- --nocapture (passed; verifies escaped load-manifest `manifest` paths are rejected before discovery)
  - WSL: cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1 -- --nocapture (passed)
  - WSL: cargo test -p zircon_runtime --lib native_loader_rejects_load_manifest_entries_outside_export_root --no-default-features --features core-min --locked --jobs 1 -- --nocapture (blocked before native-loader tests by active runtime UI text-layout test compile diagnostics that currently trigger a rustc ICE)
  - rustfmt --check zircon_runtime/tests/export_build_plan_contract.rs (passed)
  - rustfmt --check zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs zircon_runtime/src/plugin/export_build_plan/generated_files.rs zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs zircon_runtime/tests/export_build_plan_contract.rs (passed)
  - cargo fmt -p zircon_runtime --check (passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
  - cargo test -p zircon_runtime --lib native_manifest_merge_preserves_runtime_and_editor_entry_modules --locked --jobs 1 --target-dir target/codex-native-dynamic-fixture --message-format short -- --nocapture
  - cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --locked --jobs 1 --target-dir target/codex-native-dynamic-fixture --message-format short -- --nocapture
  - TDD red: cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --locked --jobs 1 --target-dir target/codex-native-abi-v2-red -- --nocapture (failed because the real fixture still loaded ABI version 1 while the new contract expected ABI version 2)
  - green: cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --locked --jobs 1 --target-dir target/codex-native-abi-v2-red -- --nocapture (1 passed; proves v2 descriptor preference, v2 runtime/editor entry names, requested capabilities, host ABI table delivery, and negotiated runtime capability diagnostics)
  - final focused verification: cargo fmt -p zircon_runtime --check (passed)
  - final focused verification: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --check (passed)
  - final focused verification: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir target/codex-native-abi-v2-plugin-check (passed)
  - final focused verification: cargo test -p zircon_runtime --lib native_loader_ --locked --jobs 1 --target-dir target/codex-native-abi-v2-red -- --nocapture (7 passed)
  - cargo test -p zircon_runtime --lib native_loader_ --locked --jobs 1 --target-dir target/codex-native-dynamic-fixture --message-format short -- --nocapture
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1 --locked
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_sound_runtime -p zircon_plugin_net_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-workspace-validation
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_support -p zircon_plugin_physics_editor -p zircon_plugin_animation_editor -p zircon_plugin_sound_editor -p zircon_plugin_net_editor -p zircon_plugin_navigation_editor -p zircon_plugin_particles_editor -p zircon_plugin_texture_editor -p zircon_plugin_virtual_geometry_editor -p zircon_plugin_hybrid_gi_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor -p zircon_plugin_native_window_hosting_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-workspace-validation
  - cargo test -p zircon_runtime project_manifest_roundtrip_preserves_plugins_and_export_profiles --lib --locked
  - cargo test -p zircon_runtime plugin_extensions --lib --locked
  - cargo test -p zircon_runtime --lib world_component_type_registry_gates_dynamic_component_attachment_when_present --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-plugin-component-registry -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_extension_registry_installs_component_types_into_world_registry --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --test-threads=1
  - cargo test -p zircon_runtime --lib tests::plugin_extensions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_extension_registry_installs_ui_components_into_runtime_registry --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-ui-components -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_extension_registry_rejects_ui_component_ids_without_plugin_prefix --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-ui-components -- --test-threads=1
  - cargo test -p zircon_runtime --lib tests::plugin_extensions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-ui-components -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_extension_registry_rejects_duplicate_module_and_render_feature_names --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-manager-duplicates -- --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_plugin_catalog_reports_duplicate_manager_contributions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-manager-duplicates -- --test-threads=1
  - cargo test -p zircon_runtime --lib tests::plugin_extensions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-manager-duplicates -- --test-threads=1
  - TDD red: cargo test -p zircon_runtime --lib registered_dynamic_component_properties_gate_editor_writes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-dynamic-component-props -- --test-threads=1 (failed before descriptor write validation because readonly label write returned Ok)
  - cargo test -p zircon_runtime --lib registered_dynamic_component_properties_gate_editor_writes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-dynamic-component-props -- --test-threads=1
  - cargo test -p zircon_runtime --lib tests::plugin_extensions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-runtime-dynamic-component-props -- --test-threads=1
  - cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 -- --nocapture (target/codex-plugin-validation)
  - cargo test -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_sound_runtime -p zircon_plugin_net_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_app --lib bootstrap_accepts_required_external_runtime_plugin_when_linked_report_contributes_module --no-default-features --features target-client --locked --jobs 1 --target-dir target/codex-app-entry-validation -- --nocapture
  - cargo test -p zircon_app bootstrap_accepts_required_native_dynamic_plugin_from_export_load_manifest --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime builtin_runtime_modules_keep_client_plugins_after_core_spine --lib --locked
  - cargo test -p zircon_runtime source_template_keeps_editor_only_plugins_out_of_runtime_registrations --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime library_embed_links_advanced_runtime_render_plugins --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime source_template_with_native_dynamic_merges_native_loader_reports --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime native_dynamic_generates_loader_manifest_without_source_template --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime native_dynamic_materialization_copies_runtime_package_without_source_crates --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime native_loader_discovers_candidates_from_export_load_manifest --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime native_loader_discovers_editor_only_native_package --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo test -p zircon_runtime optional_unavailable_runtime_plugin_stays_warning_only --lib --locked --target-dir target/codex-plugin-interface-check
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
  - cargo check --workspace --locked
  - cargo test -p zircon_editor --lib --locked -- --test-threads=1
  - cargo test -p zircon_app --locked
  - cargo test -p zircon_editor editor_plugin_toggle_refreshes_snapshot_and_view_gate --lib --locked
  - cargo check -p zircon_app --lib --no-default-features --features target-client --locked
  - cargo check -p zircon_editor --lib --locked
  - cargo check -p zircon_editor --lib --locked --message-format short --target-dir target/codex-plugin-interface-check
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-plugin-interface-check --quiet
  - cargo check -p zircon_plugin_runtime_diagnostics_editor --lib --target-dir ..\target\codex-plugin-interface-check
  - cargo check -p zircon_plugin_ui_asset_authoring_editor --lib --target-dir ..\target\codex-plugin-interface-check
  - cargo check -p zircon_plugin_native_window_hosting_editor --lib --target-dir ..\target\codex-plugin-interface-check
  - cargo check -p zircon_plugin_editor_support --locked --jobs 1 --target-dir target/codex-editor-plugin-red
  - cargo test -p zircon_plugin_editor_support -p zircon_plugin_physics_editor -p zircon_plugin_animation_editor -p zircon_plugin_sound_editor -p zircon_plugin_net_editor -p zircon_plugin_navigation_editor -p zircon_plugin_particles_editor -p zircon_plugin_texture_editor -p zircon_plugin_virtual_geometry_editor -p zircon_plugin_hybrid_gi_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor -p zircon_plugin_native_window_hosting_editor --locked --jobs 1 --target-dir target/codex-editor-plugin-red -- --nocapture
  - cargo test -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir target/codex-editor-plugin-red -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --target-dir D:\cargo-targets\zircon-render-plugin-source
  - cargo test -p zircon_runtime --lib plugin_render_feature --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_runtime --lib default_pipeline_assets_do_not_embed_pluginized_advanced_builtin_features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_runtime --lib pipeline_compile --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_runtime --lib render_framework_bridge --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib hybrid_gi_gpu --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib hybrid_gi_gpu_hierarchy --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib hybrid_gi_gpu_runtime_source --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib hybrid_gi_resolve_history --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib hybrid_gi_resolve_surface_cache --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib hybrid_gi_scene_prepare_resources --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib m5_flagship_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib virtual_geometry_execution_args_authority --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib virtual_geometry_node_and_cluster_cull_execution --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib virtual_geometry_unified_indirect --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo test -p zircon_runtime --lib virtual_geometry_execution_stats --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2 -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-cutover-2
  - cargo test -p zircon_runtime --lib source_template_preserves_builtin_catalog_target_modes_after_manifest_completion --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-export-plugin-core-min -- --nocapture
  - cargo test -p zircon_runtime --lib export_build_plan --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-export-plugin-core-min -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-export-plugin-core-min
  - cargo test -p zircon_runtime --lib export_build_plan --locked --jobs 1 --target-dir D:\cargo-targets\zircon-export-runner-convergence -- --nocapture
  - cargo test -p zircon_editor --lib native_dynamic_export_without_source_template_skips_cargo_and_writes_loader_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-export-runner-convergence -- --nocapture
  - cargo test -p zircon_editor --lib native_dynamic_export_builds_native_cargo_package_before_materializing --locked --jobs 1 --target-dir D:\cargo-targets\zircon-export-runner-convergence -- --nocapture
  - cargo check -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source
  - cargo test -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_sound_runtime -p zircon_plugin_net_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_app --lib linked_runtime_render_feature_descriptors_rebuild_default_pipelines --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_app --lib runtime_bootstrap_without_linked_virtual_geometry_keeps_base_pipeline_lightweight --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_app --lib quality_profile_capability_gates_do_not_reopen_legacy_builtin_render_features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_app --lib profile_bootstrap --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-source -- --nocapture
  - cargo test -p zircon_editor --lib editor_runtime_consumes_plugin_registration_reports_with_capability_gate --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-plugin-source -- --nocapture
  - cargo test --workspace --locked --jobs 1 --target-dir target/codex-editor-plugin-red -- --nocapture
doc_type: module-detail
---

# Runtime/Editor Pluginized Export

## Ownership

The minimal runtime/editor split is expressed through shared runtime contracts, app entry configuration, editor capability filtering, and an independent plugin workspace.

`zircon_runtime::plugin` owns serialized package, project-plugin, and export-profile data because these contracts must be consumable by runtime exports without depending on the editor crate. `zircon_app::EntryConfig` accepts the target mode, project plugin manifest, and optional export profile, then resolves the runtime module manifest before bootstrapping `CoreRuntime`. `zircon_editor` owns editor capability snapshots and editor-side plugin/export commands.

## Minimal Core Boundary

`RuntimeCoreProfile::minimal()` describes the runtime minimum: lifecycle, assets, scene, base render, and plugin loader capabilities. `GraphicsBase` is now loaded by the runtime core module list rather than selected as an optional plugin. `EditorCoreProfile::minimal()` describes the editor minimum: UI shell, asset core, scene interaction, runtime render embedding, plugin management, and capability bridge.

Optional capabilities are represented by project plugin selections and editor capability IDs. Advanced features such as physics, animation, runtime diagnostics, native window hosting, and UI asset authoring are no longer implied by the editor shell itself; views declare `required_capabilities` and the registry hides, rejects registration, or rejects opening when those capabilities are absent. The `editor.module_plugins` view is part of the minimal editor shell so module state remains reachable even when all optional editor subsystems are disabled.

The hard-cut runtime crate boundary now also reflects that split at compile time. `target-client` and `target-editor-host` no longer pull a synthetic graphics-base plugin feature. `WorldDriver` keeps the core scene tick path alive without optional physics or animation crates. Optional physics stepping, animation sequence application, sound, texture, network, navigation, particles, virtual geometry, and hybrid GI implementations no longer live under `zircon_runtime`; they are plugin package responsibilities. This makes the no-default `core-min` build a real minimum instead of a monolithic runtime build that still drags optional implementations into the library.

The second hard-cut pass removes the stale `zircon_runtime::physics` and `zircon_runtime::animation` root module declarations after those implementations moved out of the runtime body. Builtin runtime module selection no longer tries to instantiate those core modules; it reports that the implementations are externalized to `zircon_plugins/physics` and `zircon_plugins/animation`. This prevents feature-gated old paths from surviving as dead compile targets while preserving non-fatal diagnostics for manifests that still request those plugins before their external crate is linked or loaded.

The third hard-cut pass removes the former optional runtime extension subtree and the runtime Cargo feature switches that used to compile optional implementations back into the core crate. `target-server` now means a headless runtime baseline rather than "core plus built-in net"; network is selected through project plugin data like other optional runtime packages. `EntryConfig` no longer stores or converts through a separate legacy runtime plugin manifest; app startup passes `ProjectPluginManifest` directly into runtime module resolution so editor, app, export, and native-aware plugin paths share one plugin selection contract. Runtime module resolution now explicitly builds `manifest_with_mode_baseline(target, project_manifest)`: the mode baseline supplies core built-in selections such as client/editor UI, and project selections override matching IDs to enable, disable, require, or retarget plugin packaging.

Integration tests that exercise optional runtime plugins follow the same rule. Old runtime-root physics/animation contract tests were removed from the runtime crate after their implementation owners moved to `zircon_plugins`; plugin-specific contracts should live with the matching plugin workspace member.

## Project And Export Data Flow

`ProjectManifest` now carries `plugins` and `export_profiles`. `ProjectPluginManifest` is the single runtime/app/editor/export plugin selection shape; `ExportBuildPlan::from_project_manifest` resolves the enabled runtime plugin set, linked runtime crates, dynamic packages, and source-template files for a selected profile. The project plugin manifest is split into selection declarations, manifest declarations, selection builders, selection accessors, defaults, and manifest state/query behavior. The export planner is folder-backed as well: declarations, default profile selection, project-manifest resolution, and generated SourceTemplate file contents live in separate child modules so future `SourceTemplate`, `LibraryEmbed`, and `NativeDynamic` behavior can grow without turning the root export file into an umbrella implementation. SourceTemplate generation writes every project plugin selection, including disabled overrides, into `zircon_plugins.rs`. `LibraryEmbed` is the strategy that turns enabled non-dynamic runtime selections into Cargo dependencies and `runtime_plugin_registrations()` calls; linked runtime crates are deduplicated by crate name in first-seen order, so alias selections cannot emit duplicate Cargo dependency keys or duplicate `plugin_registration()` calls while project metadata still records each selection. SourceTemplate-only profiles keep the selection data but leave generated registration calls empty. If a non-dynamic selection is evaluated by a profile that enables neither `LibraryEmbed` nor `SourceTemplate`, the plan records a diagnostic because that plugin has no export carrier in the selected profile. The generated `src/main.rs` always calls `EntryRunner::bootstrap_with_runtime_plugin_registrations(...)`, so SourceTemplate+LibraryEmbed exports activate linked Rust plugin crates through `plugin_registration()` while SourceTemplate-only exports preserve manifest state without linking plugin crates. Editor-only plugins and `NativeDynamic` packages cannot synthesize nonexistent runtime crate calls.

`ExportGeneratedFile` carries `path`, `purpose`, and `contents`. For `SourceTemplate`, the generated set includes a Cargo manifest with the selected app target feature and linked plugin runtime crate dependencies, a platform `src/main.rs` entry, `src/zircon_plugins.rs` with project plugin/export profile construction code, and an `assets/zircon-project.toml` manifest copy. `NativeDynamic` selections generate `plugins/native_plugins.toml` only when the selected export profile includes the `NativeDynamic` strategy, even when the profile does not request `SourceTemplate`; a dynamic-only export therefore has an explicit loader manifest listing each copied package directory and `plugin.toml` path. Native dynamic package ids are deduplicated in first-seen order before loader manifest generation, so duplicate project selections do not emit duplicate native load rows. The planner also deduplicates by sanitized output directory: if distinct package ids such as `physics.debug` and `physics_debug` both map to `plugins/physics_debug`, the first package remains selected, the later package is skipped, and a planner diagnostic names the duplicate output directory before any load-manifest row is generated. A profile that includes the `NativeDynamic` strategy enables source-template loader wiring when dynamic selections exist, but it does not coerce `LibraryEmbed` selections into copied native packages; the per-selection `packaging` field remains authoritative and skipped dynamic selections produce planner diagnostics. The manifest keeps the original plugin id but maps the package directory through an ASCII path-component sanitizer, preventing plugin ids from becoming path traversal, nested output paths, or colliding load-manifest destinations. When a `SourceTemplate` profile includes `NativeDynamic` and at least one selected package is `NativeDynamic`, the generated `src/main.rs` locates `plugins/native_plugins.toml` by walking upward from the executable directory and current directory, loads native reports with `NativePluginLoader`, and merges them with linked Rust `runtime_plugin_registrations()` before bootstrapping.

`ExportBuildPlan::write_generated_files(root)` materializes those generated files into an export directory and creates parent directories as needed. `ExportBuildPlan::materialize(root)` wraps generated file writing in an `ExportMaterializeReport` that carries the plan diagnostics even when the selected profile generates no files, while `materialize_with_native_packages(plugin_root, output_root)` additionally copies selected `NativeDynamic` plugin packages into `output_root/plugins/<sanitized-plugin-id>`. Native dynamic materialization copies `plugin.toml`, resource directories such as `assets/` or `resources/`, and only compiled native artifacts from `native/` (`.dll`, `.so`, `.dylib`, plus symbol sidecars); runtime/editor source crates and native crate source files stay out of the runtime distribution. If a selected native package has no `native/` directory or no compiled artifact under `native/`, the materializer records a diagnostic instead of silently publishing source-only native payloads, and duplicate sanitized output directories are rejected with diagnostics. Source package lookup also avoids treating package ids with path components as direct child directories; those packages must be discovered by scanning `plugin.toml` files under the configured plugin root, which keeps ids such as `../external` from escaping the plugin root during materialization. `EditorExportBuildReport` now carries generated files, copied packages, the plan, the generated SourceTemplate Cargo invocation, native plugin Cargo invocations, stdout/stderr/status, and diagnostics; `EditorExportCargoInvocation` is re-exported with the report from `zircon_editor::ui::host` and the crate root so external callers can name the public report fields. `EditorManager::execute_native_aware_export_build(...)` stages native packages before materialization: it copies static package metadata/resources into `.native-dynamic-staging/<sanitized-plugin-id>`, reuses existing compiled native artifacts when present, and when a package exposes `native/Cargo.toml` it builds that cdylib into `.native-dynamic-build/<sanitized-plugin-id>` before publishing only the produced library into the exported package. Those staging/build roots are temporary and are removed after materialization; cleanup failures are diagnostic-only so export diagnostics capture them without aborting the minimal editor host. `EditorManager::execute_export_build(...)` and `execute_native_aware_export_build(...)` invoke Cargo against the generated manifest after materialization when a SourceTemplate `Cargo.toml` exists; build failures are reported as diagnostics instead of breaking the minimal editor host. The native-aware export runner validates the exported `plugins/native_plugins.toml` with the loader matching the export target mode: client/server profiles use runtime-only native loading, while editor-host profiles use editor-only native loading. Both editor export execution paths merge materialization diagnostics, native discovery diagnostics, target-specific native loader diagnostics, and Cargo status into one deduplicated report before writing `export-diagnostics.txt`; the diagnostics directory is created even for profiles that generate no files, so strategy mismatches and skipped-Cargo notices are still persisted for dynamic-only or intentionally empty exports.

The editor-side native preparation module is folder-backed: `prepare.rs` owns the package loop, `native_dynamic_preparation.rs` owns the temporary-root/report shape, `staging.rs` owns static package metadata/resource staging, `artifacts.rs` owns native artifact filtering and built-library copy, `cargo_build.rs` owns the native Cargo process boundary, `cleanup.rs` owns temporary directory removal, and `package_metadata.rs` owns crate-name/build-target helpers.

`NativePluginLoader` is the current native dynamic backend. It discovers `plugin.toml` packages, infers platform dynamic library names from runtime or editor plugin crate names, attempts to load existing libraries, and records missing/failed libraries as diagnostics. The report is non-fatal by design so a failed dynamic plugin never breaks the minimal runtime. Its implementation is also folder-backed: candidate/report/loaded declarations, manifest discovery, exported load-manifest discovery, platform library naming, ABI probing, and load attempts are separate behavior files under `native_plugin_loader/`.

Native dynamic plugins share a minimal ABI v2 handshake with ABI v1 fallback. A library may export `zircon_native_plugin_descriptor_v2`, returning `NativePluginAbiV2` with ABI version, plugin id, optional package-manifest TOML, optional runtime/editor entry names, and requested capabilities; if that symbol is missing, the loader falls back to `zircon_native_plugin_descriptor_v1` and `NativePluginAbiV1`. The loader copies that data into an owned `NativePluginDescriptor`; missing or invalid descriptors become diagnostics while keeping the loaded-library path non-fatal. V2 entries receive `NativePluginHostFunctionTableV2` with the ABI version, an opaque host handle, granted-capability text scoped to the entry module kind and package manifest, a host ABI query, and a host capability query. They return `NativePluginEntryReportV2` with negotiated capabilities and an optional `NativePluginBehaviorV2` behavior table, while v1 entries return `NativePluginEntryReportV1` without behavior. Entry reports can contribute package-manifest TOML and newline-delimited diagnostics; v2 behavior callbacks are only invokable through `LoadedNativePlugin`, keeping the loaded library alive while function pointers are called. ABI version mismatches, missing symbols, invalid manifest TOML, null entry pointers, and entry diagnostics remain non-fatal. This keeps dynamic loading aligned with the same package manifest and catalog contracts used by source-template and library-embed exports. `zircon_plugins/native_dynamic_fixture` is the first real cdylib fixture for this path: its native crate exports v1/v2 descriptors plus runtime/editor entry symbols, and the runtime loader test builds the actual platform library, copies it into a temporary native package, loads it through `NativePluginLoader`, and asserts descriptor ownership plus entry diagnostic merging.

The ABI v2 M0 behavior table is intentionally byte-oriented. `NativePluginBehaviorV2` declares command and event manifests as C strings, invokes commands through a command name plus `NativePluginByteSliceV2`, returns plugin-owned `NativePluginOwnedByteBufferV2` buffers that must be released through the plugin-provided `free` callback, and requires every callback to return `NativePluginCallbackStatusV2` status/diagnostics. Rust native plugins must catch their own panics before returning over this `extern "C"` boundary; the host validates the status/diagnostic result but does not attempt to recover from undefined cross-FFI unwinding. The host-owned wrapper copies returned bytes before calling plugin free, frees allocated zero-length buffers, reports malformed buffer metadata, records free failures such as allocation/free owner-token mismatches as diagnostics, and exposes save-state, restore-state, unload, and stateless behavior queries through `LoadedNativePlugin` without sharing Rust trait objects, `Arc`, borrowed references, or host-owned GPU/editor objects. The native fixture proves the current boundary with serialized echo command payloads, denied command diagnostics, panic-to-status conversion, allocation/free mismatch diagnostics, state save/restore, explicit stateless editor behavior, safe unload, and denied host capability diagnostics.

`NativePluginLoadReport` exposes discovered `plugin.toml` package manifests even when the dynamic library is missing, then merges descriptor-backed, runtime-entry-backed, and editor-entry-backed package manifests into the discovered declaration when loading succeeds. This preserves split runtime/editor module contributions instead of letting one entry report replace the other. Descriptor and entry diagnostics stay attached to the same report, so editor-side tooling can merge external dynamic packages into the same plugin status surface as built-in packages while still showing load failures. `NativePluginLoader::discover_from_load_manifest(export_root)` and `load_all_from_load_manifest(export_root)` consume the generated `plugins/native_plugins.toml` file, resolve listed package manifests relative to the export root, and produce the same candidate/report shape as directory discovery. `load_runtime_from_load_manifest(export_root)` follows the same manifest path but only loads packages that declare runtime modules and only invokes runtime entry functions for runtime startup and exported source templates. `load_editor_from_load_manifest(export_root)` and `load_discovered_editor(project_plugins)` provide the matching editor-only path for native editor registration, so editor extension registration can consume editor entry reports without invoking runtime entry functions. Target-specific native loading re-selects the expected dynamic library from the package modules before probing: runtime-only loading uses the runtime module crate name and editor-only loading uses the editor module crate name. Full diagnostic loading is explicit through `load_discovered_all(...)` / `load_all_from_load_manifest(...)`; it groups target module kinds by dynamic-library path instead of choosing only the first module crate, so a combined runtime/editor cdylib is loaded once and both entries are invoked, while split runtime/editor cdylibs are probed independently so plugin-window status can report both halves. Candidate paths keep missing libraries under the package `native/` directory instead of falling back to the package root, so diagnostics point at the same distribution layout that materialization produces. Load-manifest discovery also checks that each entry's declared `id` matches the loaded package manifest and that the loaded `plugin.toml` sits under the entry `path`, reporting non-fatal diagnostics for mismatches while still preserving the discovered package. `NativePluginLoadReport::runtime_plugin_registration_reports()` converts discovered package manifests that declare at least one runtime module into standard `RuntimePluginRegistrationReport` values, strips editor module declarations from the runtime-facing package manifest, synthesizes descriptor-backed runtime modules from package runtime module declarations, and carries only runtime-side entry diagnostics into the runtime registration report while keeping ABI v1 unchanged; editor entry diagnostics remain available through the load report and plugin-window status. `diagnostics_for_runtime_plugin(...)` and `diagnostics_for_editor_plugin(...)` make that target filtering explicit for runtime and editor registration paths, while `diagnostics_for_plugin(...)` remains the full plugin-window diagnostic surface. The report sorts and deduplicates descriptor, entry, and per-plugin diagnostic projections, which keeps split native packages from duplicating identical status lines when more than one target library contributes the same warning. Editor-only native packages remain discoverable for editor status and editor registration, but they do not enter runtime plugin registration, do not probe editor dynamic libraries in runtime-only loading, and do not touch the minimal runtime startup graph. `NativePluginLoadReport::diagnostics_for_plugin(plugin_id)` exposes per-plugin native diagnostics, so the module plugin window can show missing library, missing descriptor, failed entry calls, and entry-returned diagnostics on the affected plugin row instead of only in the global report. `EntryRunner::bootstrap_with_native_plugins_from_export_root(...)` uses the runtime-only loader through the same linked-registration path used by Rust crate plugins, so required project plugin selections can be satisfied by exported native dynamic package manifests and their runtime module declarations enter the startup module graph without invoking editor entry functions. Editor-only native packages are also discovered by falling back from runtime module crates to editor module crates when inferring the dynamic library name. `EditorManager::native_plugin_status_report(...)` discovers dynamic packages from the project plugin directory, folds native manifests into `EditorPluginStatusReport`, reflects enabled/required state from the project manifest, and preserves native loader diagnostics for the plugin window. Each plugin row now includes `package_source` and an aggregate `load_state` computed across every loaded dynamic library for that plugin, so the Slint module plugin pane can distinguish builtin catalog entries, native packages, missing libraries, load failures, entry failures, any loaded library without an ABI descriptor, loaded libraries with diagnostics, and fully loaded packages.

Load-manifest path traversal and duplicate package ids are also diagnostic-only: entries whose `manifest` or `path` escape the export root are rejected before native loading, the first valid package candidate remains active, and later duplicate package ids are ignored before dynamic-library probing. This prevents malformed load manifests from loading outside the exported distribution or probing the same dynamic package repeatedly while preserving the affected plugin diagnostics for editor and runtime status reports.

Under the current ABI v1/v2 manifest handshake plus M0 behavior table, `NativeDynamic` remains a manifest, diagnostics, export-package, load-manifest, and behavior-boundary validation backend. It can project package/module descriptors into runtime/editor status and startup reports, and the fixture now proves the C ABI mechanics for byte commands, state blobs, unload, status conversion, capability denial, and plugin-owned memory release. Linked Rust behavior still activates through `SourceTemplate` or `LibraryEmbed` generated `plugin_registration()` calls until a later migration explicitly registers callable runtime/editor operations on top of the M0 table.

Native-aware export planning is available through `EditorManager::complete_native_aware_project_plugin_manifest(...)`, `EditorManager::generate_native_aware_export_plan(...)`, and `EditorManager::execute_native_aware_export_build(...)`. These entry points keep the default builtin-catalog export path unchanged, but when a project wants NativeDynamic packages they merge native package manifests into the project plugin selections before building and materializing the `ExportBuildPlan`. Native-discovered selections default to `NativeDynamic` packaging even when the package manifest also advertises source-template or library-embed strategies, because the discovery path is explicitly representing an external dynamic package. Their default target modes are aggregated from every declared native package module, so editor-only native packages keep `editor_host` selections and combined runtime/editor cdylibs keep both sides visible before export planning. `EditorManager::native_editor_plugin_registration_reports(...)` converts discovered native package manifests into standard `EditorPluginRegistrationReport` values with editor capabilities and native diagnostics, giving editor-side dynamic packages the same owned report shape as linked editor plugin crates. ABI v2 now carries requested and negotiated capabilities through a host function table, and the M0 table carries serialized command/event declarations plus state/unload callbacks, but editor/runtime operation registration on top of those callbacks remains a later migration step.

The editor exposes this through `EditorManager`: plugin directory resolution, project plugin enable/disable mutation, export plan generation, and export build entry points. The plugin enablement path rejects attempts to disable `required` builtin or native selections, so the module plugin pane's disabled button state is backed by the same manager-side rule used by scriptable editor actions. Native-aware manifest completion and enablement use manifest discovery only, avoiding dynamic library loads while the editor is merely resolving project selections; status, native registration, and exported-loader validation remain the loading paths because they need ABI diagnostics or entry-backed manifest data. The build entry points materialize files and native packages, then run Cargo only when the materialized plan includes a generated `Cargo.toml`. Pure `NativeDynamic` exports therefore copy packages and produce `plugins/native_plugins.toml` without requiring a source-template Cargo project. When a native loader manifest was generated, the native-aware build path reads it back through the runtime-only or editor-only load-manifest API selected from the export profile target mode and folds those diagnostics into the same `EditorExportBuildReport` as source package discovery and Cargo status. The parent `editor_manager_plugins_export/mod.rs` is now a structural boundary: `reports/` owns public plugin/export report declarations with one declaration per file, `status/` owns plugin status reporting, `enablement/` owns plugin and capability toggles, `manifest_completion/` owns project manifest completion, `native_registration/` owns the native editor registration manager entry point and package-to-registration projection, `package_projection/` owns package module-crate lookup, runtime/editor capability projection, builtin project-selection projection, and native project-selection projection, and `export_build/` owns the export runner boundary. Inside `manifest_completion/`, `builtin.rs` owns builtin runtime/editor catalog completion and `native.rs` owns native-discovered package completion. Inside `enablement/`, `project.rs` owns builtin project plugin enable/disable mutation, `native.rs` owns native-aware project plugin toggles, and `capabilities.rs` owns editor capability, subsystem, and editor-plugin capability toggles. Inside `status/`, `builtin.rs` owns builtin catalog status projection, `native.rs` owns native package overlay and discovered-native status rows, and `native_load_state.rs` owns the native loader diagnostic-to-state classifier. Inside `export_build/`, `report.rs` and `cargo_invocation.rs` own the public export report declarations, `manager.rs` owns the `EditorManager` export entry points, `cargo_build.rs` owns the generated SourceTemplate Cargo process boundary, `generated_files.rs` owns generated-file probes, and `diagnostics.rs` owns Cargo/export diagnostics formatting and persistence. Platform-specific runner policy can still wrap this call, but the default editor path now has concrete materialization, optional Cargo execution, native package copying, and exported-loader-manifest validation instead of only emitting source files.

## Plugin Workspace

`zircon_plugins` is a separate Cargo workspace. The root workspace does not include it, so minimal runtime/editor development does not compile every plugin by default. The first hard-cut package set is `physics`, `sound`, `texture`, `net`, `navigation`, `particles`, `animation`, `virtual_geometry`, and `hybrid_gi`, each with split `runtime` and `editor` crates plus a `plugin.toml` package manifest. The `native_dynamic_fixture` package is intentionally different: it has a `plugin.toml` plus a single `native` cdylib crate, because its job is to validate native ABI/export loading and the M0 behavior table rather than provide a linked Rust runtime/editor implementation. The fixture exports ABI v2 symbols as the preferred path and keeps ABI v1 symbols only as the loader fallback proof; its v2 runtime behavior table covers serialized command bytes, plugin-owned buffer free, panic/status conversion, state save/restore, unload, and denied capability diagnostics, while its editor behavior declares itself stateless.

The independent workspace validation path is explicit: invoke Cargo with `--manifest-path zircon_plugins/Cargo.toml` and select the plugin packages under test with `-p`. Runtime plugin checks can validate `physics`, `animation`, `sound`, `net`, `navigation`, `particles`, `texture`, `virtual_geometry`, `hybrid_gi`, and `native_dynamic_fixture` without adding those packages to the root workspace. Editor plugin checks select `zircon_plugin_editor_support` plus the editor plugin crates and necessarily compile the current `zircon_editor` library as their host contract; warnings from active editor work are diagnostics, but a successful check still proves the plugin workspace is not being pulled through root default members.

The editor-only package set currently includes `runtime_diagnostics`, `ui_asset_authoring`, and `native_window_hosting`. These packages provide independent editor crates and `plugin.toml` declarations for capabilities already present in `EditorPluginDescriptor::builtin_catalog()`. They now implement `EditorPlugin::register_editor_extensions(...)` directly instead of only declaring capabilities: runtime diagnostics contributes its diagnostics view, UI asset authoring contributes its asset view/template surface, and native window hosting contributes the workbench and prefab window views. They also use the shared `zircon_plugin_editor_support` helper, so editor-only and runtime-backed editor crates register drawers, UI templates, menu items, open-view operations, and capability-gated views through the same standard plugin interface.

Editor-only packages are first-class builtin catalog entries even when they do not declare a runtime crate. `EditorManager::complete_project_plugin_manifest(...)` adds missing editor-only catalog selections as disabled project entries, and `EditorManager::set_project_plugin_enabled(...)` can now enable either runtime-backed builtin packages or editor-only builtin packages through the same path. Plugin status diagnostics no longer treat the absence of a runtime crate as an error for packages that only declare editor modules.

Runtime-backed plugins should contribute at least one runtime module when they need to be visible in the startup module graph. `physics`, `animation`, `sound`, and `net` now carry their migrated manager/driver implementations inside their matching runtime plugin crates and register their `ModuleDescriptor` through `RuntimePluginRegistrationReport`. `navigation`, `particles`, and `texture` are no longer descriptor-only shells: each owns a small manager-backed runtime module and a behavior test so later subsystem work has a real activation point. Every runtime plugin descriptor now declares the same target modes as its package `plugin.toml`, so descriptor-derived package manifests and project selections preserve client/server/editor availability during export and startup filtering. `virtual_geometry` and `hybrid_gi` now contribute render feature descriptors with required extract sections, capability gates, stage pass metadata, queue lanes, and frame-history binding data through the registry. The runtime renderer feature asset layer can also hold descriptor-owned plugin features through `RendererFeatureAsset::plugin(...)` and `RendererFeatureSource::Plugin(...)`, so compiled feature names, duplicate detection, and advanced runtime flags can follow descriptor names/capabilities instead of requiring every contributed feature to masquerade as a `BuiltinRenderFeature`. Advanced plugin descriptors with `VirtualGeometry`, `HybridGlobalIllumination`, or ray-tracing class capability requirements are filtered by `RenderPipelineCompileOptions::enabled_capabilities`; flagship quality profiles now enable only those capability gates, not the legacy `BuiltinRenderFeature` switches, so advanced quality flags cannot reopen old core VG/GI pass lists. Runtime submission context and scene renderer feature flags now also read VG/GI activation from descriptor capability metadata only. The old built-in VG/GI descriptor modules were removed, and `BuiltinRenderFeature::VirtualGeometry` / `BuiltinRenderFeature::GlobalIllumination` dispatch now returns inert descriptors with no passes, no executor ids, and no capability requirements, so legacy enum identity cannot create runtime state or graph work. `RenderPipelineAsset::with_plugin_render_features(...)` and `apply_plugin_render_features(...)` replace colliding advanced built-in slots by descriptor name or overlapping capability requirement before appending plugin-owned feature assets, which prevents VG/GI plugin descriptors from coexisting with their old core built-in pass lists. Linked runtime registration reports now feed those render feature descriptors into `runtime_modules_for_target_with_plugin_registration_reports(...)`; the selected `GraphicsModule` stores them and builds `WgpuRenderFramework` with plugin-aware default forward/deferred pipelines before any viewport submits a frame. The same descriptor list is passed into `SceneRenderer`, which registers descriptor-owned render pass executor ids as plugin-provided no-op executors for the current MVP. The base `RenderPassExecutorRegistry::with_builtin_noop_executors()` no longer carries VG/GI executor ids, so advanced executor coverage only appears when the matching linked plugin descriptor is present. Pipeline asset registration and reload use a validation compile that opens the asset's declared quality gates and capability requirements before executor-id validation, so a custom pipeline cannot hide a VG/GI plugin executor behind a closed quality/capability gate unless the matching linked descriptor has registered that executor id. The default forward/deferred pipeline asset constructors no longer embed the pluginized advanced VG/GI built-in slots at all; Wgpu applies linked descriptors directly, so requesting an advanced quality flag without the matching linked plugin stays on the lightweight base pipeline even when custom legacy fixtures are registered explicitly. Runtime graphics tests now share `plugin_render_feature_fixtures` for linked VG/GI descriptors, so direct `SceneRenderer` tests and `WgpuRenderFramework` tests opt into plugin render features the same way startup does instead of relying on hidden core defaults; `hybrid_gi_resolve_history`, `hybrid_gi_resolve_render`, `virtual_geometry_args_source_authority`, `virtual_geometry_submission_execution_order`, `virtual_geometry_gpu`, and `virtual_geometry_prepare_render` now follow the same descriptor capability path for their direct-renderer regression coverage. The runtime graphics test tree no longer contains legacy `with_feature_enabled(BuiltinRenderFeature::VirtualGeometry|GlobalIllumination)` activation points. As the first heavy-state migration seam, `SceneRenderer` now stores Hybrid GI / Virtual Geometry GPU readbacks plus VG debug snapshot, indirect counts, indirect buffer family, node-and-cluster-cull last-state, selected-cluster/visbuffer64/hardware-rasterization render-path last-state, execution summary, execution indirect offsets, and mesh-draw submission records inside `SceneRendererAdvancedPluginOutputs` instead of flat core fields. `SceneRendererCore` now also owns Hybrid GI, Virtual Geometry, and VG indirect-args GPU resources through `SceneRendererAdvancedPluginResources`; VG/GI runtime prepare scheduling delegates through that owner, VG indirect statistics/render-path execution enters through `collect_virtual_geometry_indirect_stats(...)`, mesh draw construction enters the VG indirect-args resource through owner-backed `build_mesh_draws(...)`, and prepare readbacks cross the render pipeline as `SceneRendererAdvancedPluginReadbacks` from folder-backed `advanced_plugin_readbacks/` with method-only access for scene-prepare resources and output collection instead of separate VG/GI pending values. Heavy graphics feedback flow and GPU-resource crate ownership remain the next migration slice.

`particles` now follows the same render-feature plugin route for its transparent particle pass. The runtime core particle descriptor module was removed, `BuiltinRenderFeature::Particle` now resolves to an externalized optional plugin descriptor with no passes until the plugin contributes its real descriptor, default forward/deferred pipeline constructors no longer embed a particle feature, and scene runtime flags enable particle rendering only when a compiled plugin feature named `particle` is present. `RenderPipelineCompileOptions` now also carries disabled plugin feature names, so `RenderQualityProfile::with_particle_rendering(false)` disables the plugin-owned `particle` feature rather than only touching the old built-in enum. The base render pass executor registry no longer registers `particle.transparent`; that executor id becomes admissible only when a linked particle render descriptor is supplied to the framework. `zircon_plugin_particles_runtime` contributes that render feature descriptor through `RuntimeExtensionRegistry::register_render_feature(...)` alongside its manager-backed module, and particle rendering tests now compare plugin-linked rendering with the profile-disabled plugin path instead of relying on in-core particle defaults.

`render_compiled_scene(...)` now returns `SceneRendererCompiledSceneOutputs`, carrying `SceneRendererAdvancedPluginReadbacks` and `VirtualGeometryIndirectStats` as one compiled-scene output package directly into `store_last_runtime_outputs(...)`. The package keeps its fields private and crosses the boundary through `new(...)` / `into_parts(...)`, which prevents callers from depending on internal field names again. `SceneRendererAdvancedPluginReadbacks` is now folder-backed: the owner declaration, Hybrid GI scene-prepare resource query, and output collection live in separate modules while the visible type name and method surface stay stable. `VirtualGeometryIndirectStats` is now folder-backed as well: owner declaration and store-parts deconstruction live separately from collection orchestration, execution segment identity, and execution-owned submission/authority buffer copying. The node-and-cluster-cull, executed-cluster-selection, hardware-rasterization, and VisBuffer64 passes now follow folder-backed shapes under `virtual_geometry_node_and_cluster_cull_pass/`, `virtual_geometry_executed_cluster_selection_pass/`, `virtual_geometry_hardware_rasterization_pass/`, and `virtual_geometry_visbuffer64_pass/`, with output/store DTOs, execution entries, typed work/selection/record/entry conversion plus packing, page-request budget handling, seed-backed selection fallback, and GPU buffer creation split into separate files; the seed-backed fallback is now a child module family for record declaration, cluster ordering, frontier rank, residency/parent fallback state, record/selection builders, and root-seed collection. Completed VG/GI readback DTOs are now folder-backed too: `VirtualGeometryGpuReadback` separates read access, completion handoff, and render-path writeback, while `HybridGiGpuReadback` separates the completion DTO from the scene-prepare resource snapshot declaration, accessors, stores, and sample queries. This removes the old ultra-wide tuple handoff and the follow-up split readback/stat parameters between core rendering and renderer output storage, so the remaining VG/GI feedback and GPU-resource migration can move owned advanced-plugin output packages instead of flat parameter lists.

The Virtual Geometry render-path naming now also follows the hard-cut rule. The former seed-backed `compat` module has been renamed to `seed_backed_execution_selection`, the non-flagship runtime mode is `BaselineGpu`, and fixed child fanout is named `FixedFanout` instead of a compatibility source. This is a terminology cutover rather than a behavior change: the current baseline GPU path still exists, but it is no longer documented or surfaced as a migration compatibility layer.

`SceneRendererAdvancedPluginOutputs` now also owns small last-output lifecycle, readback, and storage rules: `previous_virtual_geometry_node_and_cluster_cull_global_state(...)` resolves the previous VG node-and-cluster-cull global state from debug snapshot or fallback last-state, `reset(...)` clears the advanced output package, `take_hybrid_gi_gpu_readback(...)` owns Hybrid GI readback extraction, and `take_virtual_geometry_gpu_readback(...)` owns Virtual Geometry readback extraction. VG GPU readback inspection, execution-summary counts, and indirect draw count now go through method accessors on the output package, so renderer core code no longer depends on those field names directly. Virtual Geometry GPU readback itself is now stored in `VirtualGeometryReadbackOutputs`; `SceneRendererAdvancedPluginReadbacks::collect_into_outputs(...)` calls `store_virtual_geometry_gpu_readback(...)` instead of assigning the old flat output field. `store_last_runtime_outputs(...)` now creates a `VirtualGeometryLastOutputUpdate` and applies it with `store_virtual_geometry_last_outputs(...)`, keeping the long VG last-state writeback inside the advanced output owner instead of scattering it across renderer core storage code. That update contract lives in `virtual_geometry_output_updates/`, is further split into `VirtualGeometryCullOutputUpdate`, `VirtualGeometryRenderPathOutputUpdate`, and `VirtualGeometryIndirectOutputUpdate`, and the output owner applies those packages through `store_virtual_geometry_cull_outputs(...)`, `store_virtual_geometry_render_path_outputs(...)`, and `store_virtual_geometry_indirect_outputs(...)`, so the remaining VG storage contract already reflects cull, render-path/readback, and indirect/execution ownership boundaries. The output owner itself is now folder-backed as `advanced_plugin_outputs/`: the state declaration, basic readback/lifecycle accessors, and storage apply methods sit in separate child modules, while VG cull, render-path, and indirect accessor modules sit in `virtual_geometry_cull_access.rs`, `virtual_geometry_render_path_access.rs`, and `virtual_geometry_indirect_access.rs`. `VirtualGeometryCullOutputs`, `VirtualGeometryRenderPathOutputs`, and `VirtualGeometryIndirectOutputs` keep the internal VG fields grouped by the same boundaries as the update packages. Those state packages now own their read accessor and `store(...)` apply methods directly, leaving `SceneRendererAdvancedPluginOutputs` as the lifecycle/readback owner and update router rather than a second place that mutates every VG field.

Runtime-backed editor plugin crates now follow the same non-placeholder rule. `physics`, `sound`, `texture`, `net`, `navigation`, `particles`, `virtual_geometry`, and `hybrid_gi` each register an authoring view, drawer, UI template, menu item, and open-view operation. `animation` registers both sequence and graph authoring views so its existing editor activity-window ids remain capability-gated by the animation plugin. The small `zircon_plugin_editor_support` crate owns the shared authoring-surface registration helper so each plugin crate declares only its ids, labels, categories, template document, and capability.

Future migrated packages should follow the same shape:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/runtime/Cargo.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Runtime plugin crates must avoid depending on `zircon_editor`. Editor plugin crates may depend on `zircon_editor` and their matching runtime contract crate.

Native ABI fixture packages may instead use `zircon_plugins/<plugin_id>/native/Cargo.toml` with `crate-type = ["cdylib"]`. These fixture crates should keep their ABI structs self-contained or generated from an explicit ABI crate in the future; they must not pull in `zircon_editor` or optional runtime implementations just to export the C symbols.

## Runtime Extension Registry

`RuntimeExtensionRegistry` is the first strongly typed registration layer for runtime plugins. It accepts manager descriptors, module descriptors, render feature descriptors, component type descriptors, and runtime UI component descriptors before those contributions are applied to startup, render, UI, or scene runtime surfaces. Duplicate manager service names, module names, render feature names, component type ids, and UI component ids are rejected with explicit diagnostics, so plugin manifests and VM host handles can share the same id authority before `CoreRuntime::register_module(...)` sees the merged service table. Runtime UI component ids must also stay under the contributing plugin prefix, such as `weather.Ui.CloudLayerInspector`, matching the ECS component id rule. The registry is split into declaration, registration behavior, read access, and module/UI/world-application files.

Component contribution metadata follows the same boundary rule: component type declarations, component property declarations, and constructors live in separate files under `component_type_descriptor/`.

`RuntimePluginDescriptor` and `RuntimePlugin` are split into declaration, builder, package-manifest projection, project-selection projection, and trait files. This keeps native Rust plugin packages and future VM-backed plugin descriptors on the same contract without centralizing all projection behavior in one file. Descriptor-derived project selections now honor descriptor `default_packaging`: linkable Rust plugin descriptors still prefer `LibraryEmbed` when it is available, while NativeDynamic-only descriptors project a NativeDynamic default selection instead of silently falling back to LibraryEmbed.

`RuntimePlugin` is the standard Rust-facing runtime plugin interface. It exposes a `RuntimePluginDescriptor`, derives the package manifest and project selection from that descriptor, and offers a `register_runtime_extensions(...)` hook for manager/module/render-feature/component/UI runtime contributions. `RuntimePluginRegistrationReport::from_plugin(...)` is the single-plugin host-facing collection result: it carries the derived package manifest, project selection, extension registry, and diagnostics in one value. `RuntimePluginCatalog` aggregates a set of those reports into package manifests, a `ProjectPluginManifest`, a merged runtime extension registry, and catalog diagnostics. `zircon_app::BuiltinEngineEntry` and `EntryRunner` accept linked registration reports separately from serializable `EntryConfig`; required project plugins are considered available when their package id is present in target-matching linked reports, the report's module descriptors are wrapped into startup modules, and the report's render feature descriptors are passed into runtime module selection so the core graphics module can rebuild its default pipelines around linked plugin contributions. Reports whose `project_selection.target_modes` do not support the requested `RuntimeTargetMode` are ignored for availability, diagnostics, module injection, and render feature injection, so an editor-only or server-only linked crate cannot satisfy a client runtime requirement by accident. It can also complete a project plugin manifest by adding disabled catalog entries and filling missing runtime crate/target-mode data before export planning. The builtin plugin catalog is now descriptor-derived through `RuntimePluginDescriptor::builtin_catalog()` instead of hand-written package manifests. This is deliberately higher level than a native library symbol: source-template exports, library-embedded exports, and future dynamic library loaders all converge on the same descriptor and registry contract.

`EditorPlugin` mirrors that shape on the editor side. It exposes an `EditorPluginDescriptor`, attaches editor module metadata to the runtime package manifest, publishes editor capabilities, and provides `register_editor_extensions(...)` for views, drawers, menu items, component drawers, UI templates, and editor operations. `EditorPluginRegistrationReport::from_plugin(...)` is the single-plugin editor host-facing collection result: it carries the combined package manifest, capability list, editor extension registry, and diagnostics. Linked editor reports enter the live event/runtime layer through `EditorEventRuntime::register_editor_plugin_registration(...)`, which wraps the registry in an `EditorExtensionRegistration` and binds the report capabilities to contributed workbench view and editor operation descriptors. Native editor registration follows the same report shape, but only packages that declare editor modules or editor capabilities are converted into editor registration reports, and their registration package manifest is projected to editor modules before entering the editor extension registry. Runtime-only native packages remain selectable/exportable without polluting the editor extension registry. The manager keeps those descriptors registered, while descriptor listing, open, restore, reflection menu projection, component drawer lookup, UI template lookup, operation discovery, and operation invocation filter through the current `EditorCapabilitySnapshot`; disabled plugin surfaces are therefore hidden and cannot be opened or invoked, and enabling the capability makes the same registration report visible without rebuilding the editor shell. `EditorPluginCatalog` aggregates editor plugin reports into package manifests, deduplicated capabilities, a merged editor extension registry, and diagnostics for the plugin window. `EditorManager::plugin_catalog()` now composes the builtin runtime and editor plugin catalogs, so plugin-window and export consumers no longer need to hand-assemble package metadata. `EditorManager::plugin_status_report(...)` produces the plugin-window status model from the project manifest plus both catalogs, including enabled/required state, target modes, packaging strategy, runtime/editor crates, runtime/editor capabilities, and per-plugin diagnostics. `EditorManager::complete_project_plugin_manifest(...)` applies catalog defaults before export planning, so generated source templates and library-embed builds do not depend on every crate name being handwritten in `zircon-project.toml`. `EditorManager::set_project_plugin_enabled(...)` is the unified builtin plugin toggle: it updates the project plugin selection from `RuntimePluginCatalog`, applies editor capabilities from `EditorPluginCatalog`, refreshes the capability snapshot, and returns an `EditorPluginEnableReport` for UI diagnostics. `EditorManager::set_project_plugin_packaging(...)` and `set_project_plugin_target_modes(...)` update the same project selection without changing editor capability enablement, returning `EditorPluginSelectionUpdateReport` so editor controls can alter release policy independently from plugin visibility. The native-aware variants perform the same updates after completing the manifest with external native package selections. The editor plugin crate may depend on `zircon_editor`; the runtime plugin crate continues to depend only on runtime contracts.

The minimal module plugin window is now a concrete Slint pane instead of a placeholder view. `editor.module_plugins` resolves to `ViewContentKind::ModulePlugins`; the host projects `EditorPluginStatusReport` into `ModulePluginsPaneData`, displays builtin and native-discovered packages with enabled/required state, target modes, packaging strategy, crate names, runtime capabilities, editor capabilities, and diagnostics, and routes Enable/Disable button clicks through `EditorManager::set_native_aware_project_plugin_enabled(...)`. The same action path now exposes policy buttons for cycling packaging strategy and target mode presets; those actions call `set_native_aware_project_plugin_packaging(...)` and `set_native_aware_project_plugin_target_modes(...)`, then save the changed selection back to `zircon-project.toml`. Successful toggles and policy edits mark layout and presentation dirty so capability-gated view lists and plugin status rows refresh immediately. The pane body is scrollable, which keeps the full builtin/native catalog reachable as plugin packages move out of the editor core.

`PluginPackageManifest` now records declared `components` and `ui_components` next to runtime/editor crate contribution metadata. This keeps export planning separate from editor-only tooling while still letting a package advertise the ECS and runtime UI surface it contributes.

The package manifest implementation is split by responsibility: module-kind declarations, module contribution declarations, package declarations, constructors, and the built-in package catalog live in separate files. `package_manifest/mod.rs` remains a structural re-export boundary only.

Plugin component ids are full dotted names such as `weather.Component.CloudLayer`. They are intentionally not collapsed into a special `World` branch. `World` stores dynamic plugin component payloads per entity, exposes them through `dynamic_component(...)`, and routes property reads/writes through the same `ComponentPropertyPath` surface used by built-in components. Built-in components keep their existing high-performance maps; plugin components sit beside them for serialization, inspector property enumeration, and editor mutation.

`ComponentTypeRegistry` is now the scene-side type layer for plugin component descriptors. A world can register `ComponentTypeDescriptor` values from runtime plugin contributions, reject duplicate component type ids, and validate that registered ids stay under the contributing plugin namespace. `RuntimeExtensionRegistry::apply_component_types_to_world(...)` is the shared install point from plugin registration reports into an active `World`, so native Rust plugins and future VM host handles converge on the same ECS type table before entity attachment. When the registry is populated, attaching or property-writing an unregistered dynamic component is rejected; an empty registry remains permissive for legacy scene data and transitional tests. If a registered descriptor declares properties, `World::set_property(...)` also enforces that dynamic component writes target a declared property and that the property is marked `editable`; descriptors with no property list remain schemaless for early plugin experiments. The registry itself is runtime metadata and is not serialized into scene documents.

Runtime UI component contributions now install into the same `UiComponentDescriptorRegistry` used by engine UI component metadata. `RuntimeExtensionRegistry::apply_ui_components_to_registry(...)` converts the lightweight plugin declaration `{ component_id, plugin_id, ui_document }` into a runtime `UiComponentDescriptor` with `plugin-ui-component` role, `plugin_id` / `ui_document` required props, and a generic `content` slot. This keeps plugin-authored `ui.toml` widgets discoverable by the runtime UI catalog without giving plugins a special side channel or letting them overwrite built-in components.

The current dynamic component payload is `serde_json::Value` so VM and native plugin hosts can feed the same storage through stable data handles. Scalar writes use the engine `ScenePropertyValue` conversion path, which keeps editor field edits and serialized dynamic component data aligned.

## Runtime Validation Notes

The focused runtime coverage now checks three contracts:

- registry collection and duplicate diagnostics for managers, modules, render features, components, and UI components, including catalog-level duplicate manager merge diagnostics
- plugin UI component id prefix validation and `UiComponentDescriptorRegistry` installation
- package manifest round-trip of runtime/editor contributions plus component declarations
- SourceTemplate generation of linked runtime crate `plugin_registration()` calls and `EntryRunner::bootstrap_with_runtime_plugin_registrations(...)`
- app startup acceptance of a required external runtime plugin when a linked registration report contributes the requested module
- app startup rejection of a linked registration report whose target modes do not match the requested runtime target
- plugin workspace runtime-package behavior tests for physics, animation, sound, net, navigation, particles, texture, virtual geometry, and hybrid GI, including render feature pass/capability metadata for the advanced render plugins
- descriptor-derived package manifests for runtime plugins preserve the target modes declared in each package `plugin.toml`
- descriptor-derived project selections preserve non-LibraryEmbed default packaging when the descriptor only advertises another export strategy such as `NativeDynamic`
- runtime-only native loading filters out editor-only packages before dynamic-library probing and does not invoke editor entry functions during runtime export startup
- native editor registration filters out runtime-only packages and uses editor-only native loading while preserving runtime-only packages for plugin status and export selection
- split native packages use the runtime module crate for runtime-only loading, the editor module crate for editor-only loading, and both target libraries during full diagnostic status loading
- native-aware export validation probes exported dynamic packages with the loader matching the export profile target mode
- runtime plugin registration reports keep editor entry diagnostics out of runtime startup diagnostics
- native runtime/editor registration paths use target-specific diagnostic report APIs instead of the full plugin-window diagnostic surface
- native diagnostic report coverage asserts that full diagnostics include both runtime/editor entry messages while target-specific projections include only their matching entry kind
- native runtime registration reports strip editor module declarations before entering runtime startup
- native editor registration reports strip runtime module declarations before entering the editor extension registry
- NativeDynamic materialization sanitizes output directories and refuses direct package-id source lookup when the id contains path components
- SourceTemplate export generation after `RuntimePluginCatalog::builtin().complete_project_manifest(...)` preserves builtin catalog target modes in the generated `src/zircon_plugins.rs` project plugin selections
- renderer feature asset behavior for descriptor-owned plugin features, duplicate plugin feature names, and capability-driven advanced runtime flags
- default forward/deferred pipeline assets excluding pluginized advanced VG/GI/particle built-in feature slots at construction time, with legacy built-in behavior isolated to explicit test fixtures
- runtime submission and scene renderer runtime flags ignoring legacy VG/GI built-in identity unless the compiled descriptor declares the matching capability requirement
- render pass executor registry coverage excluding VG/GI/particle from the base built-in executor list and accepting those executor ids only through linked plugin render descriptors
- pipeline asset registration/reload rejection of plugin executor ids when the asset declares them behind quality or capability gates but the framework was not constructed with the matching linked render descriptor, plus acceptance when the descriptor is linked
- app startup render submission with a linked `virtual_geometry` registration report, proving contributed render feature descriptors rebuild the default pipeline and replace the old core virtual-geometry pass list before execution stats are recorded
- app startup render submission without linked advanced render plugins, proving a VG/GI quality request no longer executes old core advanced passes from the base pipeline
- app startup render submission with an explicitly registered legacy built-in pipeline fixture, proving VG/GI quality flags only open capability-gated plugin descriptors and do not reopen old `BuiltinRenderFeature` pass lists
- scene `ComponentTypeRegistry` duplicate/type-prefix diagnostics and dynamic component attachment gate when registered descriptors are present
- RuntimeExtensionRegistry component id prefix validation and World ComponentTypeRegistry installation
- dynamic plugin component attach, world serialization round-trip, property read, editable property write, readonly write rejection, and undeclared property rejection through existing scene property paths
- plugin unload guard behavior: `World::ensure_plugin_components_can_unload(plugin_id)` rejects unload while any entity still owns a dynamic component whose id starts with that plugin namespace, and succeeds after those components are removed
- target-client module ordering keeps GraphicsBase inside the minimal runtime core and appends UI as the default client plugin, matching the compile-time feature split
