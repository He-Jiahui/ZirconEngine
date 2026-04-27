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
  - zircon_runtime/src/plugin/runtime_plugin/mod.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/project_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/Cargo.toml
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
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks.rs
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
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation.rs
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
  - zircon_plugins/native_dynamic_sample/plugin.toml
  - zircon_plugins/native_dynamic_sample/native/Cargo.toml
  - zircon_plugins/native_dynamic_sample/native/src/lib.rs
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
  - zircon_runtime/src/plugin/runtime_plugin/mod.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/project_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/Cargo.toml
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
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation.rs
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
  - zircon_plugins/native_dynamic_sample/plugin.toml
  - zircon_plugins/native_dynamic_sample/native/Cargo.toml
  - zircon_plugins/native_dynamic_sample/native/src/lib.rs
plan_sources:
  - user: 2026-04-27 Native Dynamic 真库样例闭环
  - user: 2026-04-27 zircon_plugins 全量插件化收敛规划
  - docs/superpowers/specs/2026-04-27-native-dynamic-sample-closure-design.md
  - docs/superpowers/plans/2026-04-27-native-dynamic-sample-closure.md
  - user: 2026-04-26 Runtime/Editor 最小本体与发行导出插件化设计
  - .codex/plans/Runtime_Editor 最小本体与发行导出插件化设计.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_sample_native --locked --jobs 1
  - cargo test -p zircon_runtime --lib native_loader_calls_real_sample_descriptor_and_entries --locked --jobs 1 --target-dir target/codex-native-dynamic-sample --message-format short -- --nocapture
  - cargo test -p zircon_runtime --lib native_loader_ --locked --jobs 1 --target-dir target/codex-native-dynamic-sample --message-format short -- --nocapture
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1 --locked
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_sound_runtime -p zircon_plugin_net_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_native_dynamic_sample_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-workspace-validation
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_support -p zircon_plugin_physics_editor -p zircon_plugin_animation_editor -p zircon_plugin_sound_editor -p zircon_plugin_net_editor -p zircon_plugin_navigation_editor -p zircon_plugin_particles_editor -p zircon_plugin_texture_editor -p zircon_plugin_virtual_geometry_editor -p zircon_plugin_hybrid_gi_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor -p zircon_plugin_native_window_hosting_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-workspace-validation
  - cargo test -p zircon_runtime project_manifest_roundtrip_preserves_plugins_and_export_profiles --lib --locked
  - cargo test -p zircon_runtime plugin_extensions --lib --locked
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

The hard-cut runtime crate boundary now also reflects that split at compile time. `target-client` and `target-editor-host` no longer pull a synthetic graphics-base plugin feature. `WorldDriver` keeps the core scene tick path alive without optional physics or animation crates. Optional physics stepping, animation sequence application, sound, texture, network, navigation, particles, virtual geometry, and hybrid GI implementations no longer live under `zircon_runtime`; they are plugin package responsibilities. This makes the no-default `core-min` build a real minimum instead of a static build that still drags optional runtime implementations into the library.

The second hard-cut pass removes the stale `zircon_runtime::physics` and `zircon_runtime::animation` root module declarations after those implementations moved out of the runtime body. Builtin runtime module selection no longer tries to instantiate those core modules; it reports that the implementations are externalized to `zircon_plugins/physics` and `zircon_plugins/animation`. This prevents feature-gated old paths from surviving as dead compile targets while preserving non-fatal diagnostics for manifests that still request those plugins before their external crate is linked or loaded.

The third hard-cut pass removes the former optional runtime extension subtree and the runtime Cargo feature switches that used to compile optional implementations back into the core crate. `target-server` now means a headless runtime baseline rather than "core plus built-in net"; network is selected through project plugin data like other optional runtime packages. `EntryConfig` no longer stores or converts through a separate legacy runtime plugin manifest; app startup passes `ProjectPluginManifest` directly into runtime module resolution so editor, app, export, and native-aware plugin paths share one plugin selection contract. Runtime module resolution now explicitly builds `manifest_with_mode_baseline(target, project_manifest)`: the mode baseline supplies core built-in selections such as client/editor UI, and project selections override matching IDs to enable, disable, require, or retarget plugin packaging.

Integration tests that exercise optional runtime plugins follow the same rule. Old runtime-root physics/animation contract tests were removed from the runtime crate after their implementation owners moved to `zircon_plugins`; plugin-specific contracts should live with the matching plugin workspace member.

## Project And Export Data Flow

`ProjectManifest` now carries `plugins` and `export_profiles`. `ProjectPluginManifest` is the single runtime/app/editor/export plugin selection shape; `ExportBuildPlan::from_project_manifest` resolves the enabled runtime plugin set, linked runtime crates, dynamic packages, and source-template files for a selected profile. The project plugin manifest is split into selection declarations, manifest declarations, selection builders, selection accessors, defaults, and manifest state/query behavior. The export planner is folder-backed as well: declarations, default profile selection, project-manifest resolution, and generated SourceTemplate file contents live in separate child modules so future `SourceTemplate`, `LibraryEmbed`, and `NativeDynamic` behavior can grow without turning the root export file into an umbrella implementation. SourceTemplate generation writes every project plugin selection, including disabled overrides, into `zircon_plugins.rs`; linked crate dependencies and `runtime_plugin_registrations()` are generated only for enabled non-dynamic runtime selections. The generated `src/main.rs` calls `EntryRunner::bootstrap_with_runtime_plugin_registrations(...)`, so SourceTemplate and LibraryEmbed exports activate linked Rust plugin crates through `plugin_registration()` instead of only serializing manifests. Editor-only plugins and `NativeDynamic` packages cannot synthesize nonexistent runtime crate calls.

`ExportGeneratedFile` carries `path`, `purpose`, and `contents`. For `SourceTemplate`, the generated set includes a Cargo manifest with the selected app target feature and linked plugin runtime crate dependencies, a platform `src/main.rs` entry, `src/zircon_plugins.rs` with project plugin/export profile construction code, and an `assets/zircon-project.toml` manifest copy. `NativeDynamic` selections also generate `plugins/native_plugins.toml`, even when the profile does not request `SourceTemplate`, so a dynamic-only export has an explicit loader manifest listing each copied package directory and `plugin.toml` path. When a `SourceTemplate` profile includes `NativeDynamic`, the generated `src/main.rs` locates `plugins/native_plugins.toml` by walking upward from the executable directory and current directory, loads native reports with `NativePluginLoader`, and merges them with linked Rust `runtime_plugin_registrations()` before bootstrapping.

`ExportBuildPlan::write_generated_files(root)` materializes those generated files into an export directory and creates parent directories as needed. `ExportBuildPlan::materialize(root)` wraps generated file writing in an `ExportMaterializeReport`, while `materialize_with_native_packages(plugin_root, output_root)` additionally copies selected `NativeDynamic` plugin packages into `output_root/plugins/<plugin_id>`. Native dynamic materialization copies `plugin.toml`, resource directories such as `assets/` or `resources/`, and only compiled native artifacts from `native/` (`.dll`, `.so`, `.dylib`, plus symbol sidecars); runtime/editor source crates and native crate source files stay out of the runtime distribution. If a selected native package has no `native/` directory or no compiled artifact under `native/`, the materializer records a diagnostic instead of silently publishing source-only native payloads. `EditorExportBuildReport` now carries generated files, copied packages, the plan, the generated SourceTemplate Cargo invocation, native plugin Cargo invocations, stdout/stderr/status, and diagnostics. `EditorManager::execute_native_aware_export_build(...)` stages native packages before materialization: it copies static package metadata/resources into `.native-dynamic-staging`, reuses existing compiled native artifacts when present, and when a package exposes `native/Cargo.toml` it builds that cdylib into `.native-dynamic-build` before publishing only the produced library into the exported package. Those staging/build roots are temporary and are removed after materialization; cleanup failures are diagnostic-only so export diagnostics capture them without aborting the minimal editor host. `EditorManager::execute_export_build(...)` and `execute_native_aware_export_build(...)` invoke Cargo against the generated manifest after materialization when a SourceTemplate `Cargo.toml` exists; build failures are reported as diagnostics instead of breaking the minimal editor host. Both editor export execution paths also write the collected diagnostics to `export-diagnostics.txt` in the output root, including skipped-Cargo notices for pure `NativeDynamic` exports and native loader failures such as missing dynamic libraries.

`NativePluginLoader` is the current native dynamic backend skeleton. It discovers `plugin.toml` packages, infers platform dynamic library names from runtime or editor plugin crate names, attempts to load existing libraries, and records missing/failed libraries as diagnostics. The report is non-fatal by design so a failed dynamic plugin never breaks the minimal runtime. Its implementation is also folder-backed: candidate/report/loaded declarations, manifest discovery, exported load-manifest discovery, platform library naming, ABI probing, and load attempts are separate behavior files under `native_plugin_loader/`.

Native dynamic plugins share a minimal ABI v1 handshake. A library may export `zircon_native_plugin_descriptor_v1`, returning a `NativePluginAbiV1` with ABI version, plugin id, optional package-manifest TOML, and optional runtime/editor entry names. The loader copies that data into an owned `NativePluginDescriptor`; missing or invalid descriptors become diagnostics while keeping the loaded-library path non-fatal. When runtime/editor entry names are present, the loader calls those symbols and converts their `NativePluginEntryReportV1` values into owned `NativePluginEntryReport` values. Entry reports can contribute package-manifest TOML and newline-delimited diagnostics. ABI version mismatches, missing symbols, invalid manifest TOML, null entry pointers, and entry diagnostics remain non-fatal. This keeps dynamic loading aligned with the same package manifest and catalog contracts used by source-template and library-embed exports. `zircon_plugins/native_dynamic_sample` is the first real cdylib fixture for this path: its native crate exports the descriptor plus runtime/editor entry symbols, and the runtime loader test builds the actual platform library, copies it into a temporary native package, loads it through `NativePluginLoader`, and asserts descriptor ownership plus entry diagnostic merging.

`NativePluginLoadReport` exposes discovered `plugin.toml` package manifests even when the dynamic library is missing, then lets entry-backed or descriptor-backed manifests override those discovered declarations when loading succeeds. Descriptor and entry diagnostics stay attached to the same report, so editor-side tooling can merge external dynamic packages into the same plugin status surface as built-in packages while still showing load failures. `NativePluginLoader::discover_from_load_manifest(export_root)` and `load_from_load_manifest(export_root)` consume the generated `plugins/native_plugins.toml` file, resolve listed package manifests relative to the export root, and produce the same candidate/report shape as directory discovery. `NativePluginLoadReport::runtime_plugin_registration_reports()` converts discovered package manifests into standard `RuntimePluginRegistrationReport` values, synthesizes descriptor-backed runtime modules from package runtime module declarations, and carries each plugin's native diagnostics into the registration report while keeping ABI v1 unchanged. `NativePluginLoadReport::diagnostics_for_plugin(plugin_id)` exposes per-plugin native diagnostics, so the module plugin window can show missing library, missing descriptor, failed entry calls, and entry-returned diagnostics on the affected plugin row instead of only in the global report. `EntryRunner::bootstrap_with_native_plugins_from_export_root(...)` uses that report through the same linked-registration path used by Rust crate plugins, so required project plugin selections can be satisfied by exported native dynamic package manifests and their runtime module declarations enter the startup module graph. Editor-only native packages are also discovered by falling back from runtime module crates to editor module crates when inferring the dynamic library name. `EditorManager::native_plugin_status_report(...)` discovers dynamic packages from the project plugin directory, folds native manifests into `EditorPluginStatusReport`, reflects enabled/required state from the project manifest, and preserves native loader diagnostics for the plugin window. Each plugin row now includes `package_source` and `load_state`, so the Slint module plugin pane can distinguish builtin catalog entries, native packages, missing libraries, load failures, entry failures, loaded libraries with diagnostics, and loaded libraries without ABI descriptors.

Native-aware export planning is available through `EditorManager::complete_native_aware_project_plugin_manifest(...)`, `EditorManager::generate_native_aware_export_plan(...)`, and `EditorManager::execute_native_aware_export_build(...)`. These entry points keep the default builtin-catalog export path unchanged, but when a project wants NativeDynamic packages they merge native package manifests into the project plugin selections before building and materializing the `ExportBuildPlan`. Native-discovered selections default to `NativeDynamic` packaging even when the package manifest also advertises source-template or library-embed strategies, because the discovery path is explicitly representing an external dynamic package. `EditorManager::native_editor_plugin_registration_reports(...)` converts discovered native package manifests into standard `EditorPluginRegistrationReport` values with editor capabilities and native diagnostics, giving editor-side dynamic packages the same owned report shape as linked editor plugin crates while the current ABI still limits native entry contributions to manifest/diagnostic data.

The editor exposes this through `EditorManager`: plugin directory resolution, project plugin enable/disable mutation, export plan generation, and export build entry points. The plugin enablement path rejects attempts to disable `required` builtin or native selections, so the module plugin pane's disabled button state is backed by the same manager-side rule used by scriptable editor actions. The build entry points materialize files and native packages, then run Cargo only when the materialized plan includes a generated `Cargo.toml`. Pure `NativeDynamic` exports therefore copy packages and produce `plugins/native_plugins.toml` without requiring a source-template Cargo project. When a native loader manifest was generated, the native-aware build path reads it back through `NativePluginLoader::load_from_load_manifest(...)` and folds those diagnostics into the same `EditorExportBuildReport` as source package discovery and Cargo status. The editor-side export runner/report boundary lives in `editor_manager_plugins_export/export_build.rs`, while the parent `editor_manager_plugins_export.rs` remains the plugin status, enablement, and manifest-completion façade. Platform-specific runner policy can still wrap this call, but the default editor path now has concrete materialization, optional Cargo execution, native package copying, and exported-loader-manifest validation instead of only emitting source files.

## Plugin Workspace

`zircon_plugins` is a separate Cargo workspace. The root workspace does not include it, so minimal runtime/editor development does not compile every plugin by default. The first hard-cut package set is `physics`, `sound`, `texture`, `net`, `navigation`, `particles`, `animation`, `virtual_geometry`, and `hybrid_gi`, each with split `runtime` and `editor` crates plus a `plugin.toml` package manifest. The `native_dynamic_sample` package is intentionally different: it has a `plugin.toml` plus a single `native` cdylib crate, because its job is to validate native ABI/export loading rather than provide a linked Rust runtime/editor implementation.

The independent workspace validation path is explicit: invoke Cargo with `--manifest-path zircon_plugins/Cargo.toml` and select the plugin packages under test with `-p`. Runtime plugin checks can validate `physics`, `animation`, `sound`, `net`, `navigation`, `particles`, `texture`, `virtual_geometry`, `hybrid_gi`, and `native_dynamic_sample` without adding those packages to the root workspace. Editor plugin checks select `zircon_plugin_editor_support` plus the editor plugin crates and necessarily compile the current `zircon_editor` library as their host contract; warnings from active editor work are diagnostics, but a successful check still proves the plugin workspace is not being pulled through root default members.

The editor-only package set currently includes `runtime_diagnostics`, `ui_asset_authoring`, and `native_window_hosting`. These packages provide independent editor crates and `plugin.toml` declarations for capabilities already present in `EditorPluginDescriptor::builtin_catalog()`. They now implement `EditorPlugin::register_editor_extensions(...)` directly instead of only declaring capabilities: runtime diagnostics contributes its diagnostics view, UI asset authoring contributes its asset view/template surface, and native window hosting contributes the workbench and prefab window views. They also use the shared `zircon_plugin_editor_support` helper, so editor-only and runtime-backed editor crates register drawers, UI templates, menu items, open-view operations, and capability-gated views through the same standard plugin interface.

Editor-only packages are first-class builtin catalog entries even when they do not declare a runtime crate. `EditorManager::complete_project_plugin_manifest(...)` adds missing editor-only catalog selections as disabled project entries, and `EditorManager::set_project_plugin_enabled(...)` can now enable either runtime-backed builtin packages or editor-only builtin packages through the same path. Plugin status diagnostics no longer treat the absence of a runtime crate as an error for packages that only declare editor modules.

Runtime-backed plugins should contribute at least one runtime module when they need to be visible in the startup module graph. `physics`, `animation`, `sound`, and `net` now carry their migrated manager/driver implementations inside their matching runtime plugin crates and register their `ModuleDescriptor` through `RuntimePluginRegistrationReport`. `navigation`, `particles`, and `texture` are no longer descriptor-only shells: each owns a small manager-backed runtime module and a behavior test so later subsystem work has a real activation point. Every runtime plugin descriptor now declares the same target modes as its package `plugin.toml`, so descriptor-derived package manifests and project selections preserve client/server/editor availability during export and startup filtering. `virtual_geometry` and `hybrid_gi` now contribute render feature descriptors with required extract sections, capability gates, stage pass metadata, queue lanes, and frame-history binding data through the registry. The runtime renderer feature asset layer can also hold descriptor-owned plugin features through `RendererFeatureAsset::plugin(...)` and `RendererFeatureSource::Plugin(...)`, so compiled feature names, duplicate detection, and advanced runtime flags can follow descriptor names/capabilities instead of requiring every contributed feature to masquerade as a `BuiltinRenderFeature`. Advanced plugin descriptors with `VirtualGeometry`, `HybridGlobalIllumination`, or ray-tracing class capability requirements are filtered by `RenderPipelineCompileOptions::enabled_capabilities`; flagship quality profiles now enable only those capability gates, not the legacy `BuiltinRenderFeature` switches, so advanced quality flags cannot reopen old core VG/GI pass lists. Runtime submission context and scene renderer feature flags now also read VG/GI activation from descriptor capability metadata only. The old built-in VG/GI descriptor modules were removed, and `BuiltinRenderFeature::VirtualGeometry` / `BuiltinRenderFeature::GlobalIllumination` dispatch now returns inert descriptors with no passes, no executor ids, and no capability requirements, so legacy enum identity cannot create runtime state or graph work. `RenderPipelineAsset::with_plugin_render_features(...)` and `apply_plugin_render_features(...)` replace colliding advanced built-in slots by descriptor name or overlapping capability requirement before appending plugin-owned feature assets, which prevents VG/GI plugin descriptors from coexisting with their old core built-in pass lists. Linked runtime registration reports now feed those render feature descriptors into `runtime_modules_for_target_with_plugin_registration_reports(...)`; the selected `GraphicsModule` stores them and builds `WgpuRenderFramework` with plugin-aware default forward/deferred pipelines before any viewport submits a frame. The same descriptor list is passed into `SceneRenderer`, which registers descriptor-owned render pass executor ids as plugin-provided no-op executors for the current MVP. The base `RenderPassExecutorRegistry::with_builtin_noop_executors()` no longer carries VG/GI executor ids, so advanced executor coverage only appears when the matching linked plugin descriptor is present. Pipeline asset registration and reload use a validation compile that opens the asset's declared quality gates and capability requirements before executor-id validation, so a custom pipeline cannot hide a VG/GI plugin executor behind a closed quality/capability gate unless the matching linked descriptor has registered that executor id. The default forward/deferred pipeline asset constructors no longer embed the pluginized advanced VG/GI built-in slots at all; Wgpu applies linked descriptors directly, so requesting an advanced quality flag without the matching linked plugin stays on the lightweight base pipeline even when custom legacy fixtures are registered explicitly. Runtime graphics tests now share `plugin_render_feature_fixtures` for linked VG/GI descriptors, so direct `SceneRenderer` tests and `WgpuRenderFramework` tests opt into plugin render features the same way startup does instead of relying on hidden core defaults; `hybrid_gi_resolve_history`, `hybrid_gi_resolve_render`, `virtual_geometry_args_source_authority`, `virtual_geometry_submission_execution_order`, `virtual_geometry_gpu`, and `virtual_geometry_prepare_render` now follow the same descriptor capability path for their direct-renderer regression coverage. The runtime graphics test tree no longer contains legacy `with_feature_enabled(BuiltinRenderFeature::VirtualGeometry|GlobalIllumination)` activation points. As the first heavy-state migration seam, `SceneRenderer` now stores Hybrid GI / Virtual Geometry GPU readbacks plus VG debug snapshot, indirect counts, indirect buffer family, node-and-cluster-cull last-state, selected-cluster/visbuffer64/hardware-rasterization render-path last-state, execution summary, execution indirect offsets, and mesh-draw submission records inside `SceneRendererAdvancedPluginOutputs` instead of flat core fields. `SceneRendererCore` now also owns Hybrid GI, Virtual Geometry, and VG indirect-args GPU resources through `SceneRendererAdvancedPluginResources`; VG/GI runtime prepare scheduling delegates through that owner, VG indirect statistics/render-path execution enters through `collect_virtual_geometry_indirect_stats(...)`, mesh draw construction enters the VG indirect-args resource through owner-backed `build_mesh_draws(...)`, and prepare readbacks cross the render pipeline as `SceneRendererAdvancedPluginReadbacks` from `advanced_plugin_readbacks.rs` with method-only access for scene-prepare resources and output collection instead of separate VG/GI pending values. Heavy graphics feedback flow and GPU-resource crate ownership remain the next migration slice.

`render_compiled_scene(...)` now returns `SceneRendererCompiledSceneOutputs`, carrying `SceneRendererAdvancedPluginReadbacks` and `VirtualGeometryIndirectStats` as one compiled-scene output package directly into `store_last_runtime_outputs(...)`. The package keeps its fields private and crosses the boundary through `new(...)` / `into_parts(...)`, which prevents callers from depending on internal field names again. This removes the old ultra-wide tuple handoff and the follow-up split readback/stat parameters between core rendering and renderer output storage, so the remaining VG/GI feedback and GPU-resource migration can move an owned advanced-plugin output package instead of a flat parameter list.

`SceneRendererAdvancedPluginOutputs` now also owns small last-output lifecycle, readback, and storage rules: `previous_virtual_geometry_node_and_cluster_cull_global_state(...)` resolves the previous VG node-and-cluster-cull global state from debug snapshot or fallback last-state, `reset(...)` clears the advanced output package, `take_hybrid_gi_gpu_readback(...)` owns Hybrid GI readback extraction, and `take_virtual_geometry_gpu_readback(...)` owns Virtual Geometry readback extraction. VG GPU readback inspection, execution-summary counts, and indirect draw count now go through method accessors on the output package, so the renderer facade no longer depends on those field names directly. `store_last_runtime_outputs(...)` now creates a `VirtualGeometryLastOutputUpdate` and applies it with `store_virtual_geometry_last_outputs(...)`, keeping the long VG last-state writeback inside the advanced output owner instead of scattering it across renderer core storage code. That update is further split into `VirtualGeometryCullOutputUpdate`, `VirtualGeometryRenderPathOutputUpdate`, and `VirtualGeometryIndirectOutputUpdate`, and the output owner applies them through `store_virtual_geometry_cull_outputs(...)`, `store_virtual_geometry_render_path_outputs(...)`, and `store_virtual_geometry_indirect_outputs(...)`, so the remaining VG storage contract already reflects cull, render-path/readback, and indirect/execution ownership boundaries.

Runtime-backed editor plugin crates now follow the same non-placeholder rule. `physics`, `sound`, `texture`, `net`, `navigation`, `particles`, `virtual_geometry`, and `hybrid_gi` each register an authoring view, drawer, UI template, menu item, and open-view operation. `animation` registers both sequence and graph authoring views so its existing editor activity-window ids remain capability-gated by the animation plugin. The small `zircon_plugin_editor_support` crate owns the shared authoring-surface registration helper so each plugin crate declares only its ids, labels, categories, template document, and capability.

Future migrated packages should follow the same shape:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/runtime/Cargo.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Runtime plugin crates must avoid depending on `zircon_editor`. Editor plugin crates may depend on `zircon_editor` and their matching runtime contract crate.

Native ABI sample packages may instead use `zircon_plugins/<plugin_id>/native/Cargo.toml` with `crate-type = ["cdylib"]`. These sample crates should keep their ABI structs self-contained or generated from an explicit ABI crate in the future; they must not pull in `zircon_editor` or optional runtime implementations just to export the C symbols.

## Runtime Extension Registry

`RuntimeExtensionRegistry` is the first strongly typed registration layer for runtime plugins. It accepts manager descriptors, module descriptors, render feature descriptors, component type descriptors, and runtime UI component descriptors before those contributions are applied to startup, render, or scene runtime surfaces. Duplicate module names, render feature names, component type ids, and UI component ids are rejected with explicit diagnostics, so plugin manifests and VM host handles can share the same id authority. The registry is split into declaration, registration behavior, read access, and module-application files.

Component contribution metadata follows the same boundary rule: component type declarations, component property declarations, and constructors live in separate files under `component_type_descriptor/`.

`RuntimePluginDescriptor` and `RuntimePlugin` are split into declaration, builder, package-manifest projection, project-selection projection, and trait files. This keeps native Rust plugin packages and future VM-backed plugin descriptors on the same contract without centralizing all projection behavior in one file.

`RuntimePlugin` is the standard Rust-facing runtime plugin interface. It exposes a `RuntimePluginDescriptor`, derives the package manifest and project selection from that descriptor, and offers a `register_runtime_extensions(...)` hook for manager/module/render-feature/component/UI runtime contributions. `RuntimePluginRegistrationReport::from_plugin(...)` is the single-plugin host-facing collection result: it carries the derived package manifest, project selection, extension registry, and diagnostics in one value. `RuntimePluginCatalog` aggregates a set of those reports into package manifests, a `ProjectPluginManifest`, a merged runtime extension registry, and catalog diagnostics. `zircon_app::BuiltinEngineEntry` and `EntryRunner` accept linked registration reports separately from serializable `EntryConfig`; required project plugins are considered available when their package id is present in target-matching linked reports, the report's module descriptors are wrapped into startup modules, and the report's render feature descriptors are passed into runtime module selection so the core graphics module can rebuild its default pipelines around linked plugin contributions. Reports whose `project_selection.target_modes` do not support the requested `RuntimeTargetMode` are ignored for availability, diagnostics, module injection, and render feature injection, so an editor-only or server-only linked crate cannot satisfy a client runtime requirement by accident. It can also complete a project plugin manifest by adding disabled catalog entries and filling missing runtime crate/target-mode data before export planning. The builtin plugin catalog is now descriptor-derived through `RuntimePluginDescriptor::builtin_catalog()` instead of hand-written package manifests. This is deliberately higher level than a native library symbol: source-template exports, library-embedded exports, and future dynamic library loaders all converge on the same descriptor and registry contract.

`EditorPlugin` mirrors that shape on the editor side. It exposes an `EditorPluginDescriptor`, attaches editor module metadata to the runtime package manifest, publishes editor capabilities, and provides `register_editor_extensions(...)` for views, drawers, menu items, component drawers, UI templates, and editor operations. `EditorPluginRegistrationReport::from_plugin(...)` is the single-plugin editor host-facing collection result: it carries the combined package manifest, capability list, editor extension registry, and diagnostics. Linked editor reports enter the live event/runtime layer through `EditorEventRuntime::register_editor_plugin_registration(...)`, which wraps the registry in an `EditorExtensionRegistration` and binds the report capabilities to contributed workbench view and editor operation descriptors. The manager keeps those descriptors registered, while descriptor listing, open, restore, reflection menu projection, component drawer lookup, UI template lookup, operation discovery, and operation invocation filter through the current `EditorCapabilitySnapshot`; disabled plugin surfaces are therefore hidden and cannot be opened or invoked, and enabling the capability makes the same registration report visible without rebuilding the editor shell. `EditorPluginCatalog` aggregates editor plugin reports into package manifests, deduplicated capabilities, a merged editor extension registry, and diagnostics for the plugin window. `EditorManager::plugin_catalog()` now composes the builtin runtime and editor plugin catalogs, so plugin-window and export consumers no longer need to hand-assemble package metadata. `EditorManager::plugin_status_report(...)` produces the plugin-window status model from the project manifest plus both catalogs, including enabled/required state, runtime/editor crates, editor capabilities, and per-plugin diagnostics. `EditorManager::complete_project_plugin_manifest(...)` applies catalog defaults before export planning, so generated source templates and library-embed builds do not depend on every crate name being handwritten in `zircon-project.toml`. `EditorManager::set_project_plugin_enabled(...)` is the unified builtin plugin toggle: it updates the project plugin selection from `RuntimePluginCatalog`, applies editor capabilities from `EditorPluginCatalog`, refreshes the capability snapshot, and returns an `EditorPluginEnableReport` for UI diagnostics. `EditorManager::set_native_aware_project_plugin_enabled(...)` extends the same project-manifest mutation path to native-discovered packages by completing the manifest with native package selections first. The editor plugin crate may depend on `zircon_editor`; the runtime plugin crate continues to depend only on runtime contracts.

The minimal module plugin window is now a concrete Slint pane instead of a placeholder view. `editor.module_plugins` resolves to `ViewContentKind::ModulePlugins`; the host projects `EditorPluginStatusReport` into `ModulePluginsPaneData`, displays builtin and native-discovered packages with enabled/required state, crate names, capabilities, and diagnostics, and routes Enable/Disable button clicks through `EditorManager::set_native_aware_project_plugin_enabled(...)`. Successful toggles are saved back to `zircon-project.toml`, then the editor marks layout and presentation dirty so capability-gated view lists refresh immediately.

`PluginPackageManifest` now records declared `components` and `ui_components` next to runtime/editor crate contribution metadata. This keeps export planning separate from editor-only tooling while still letting a package advertise the ECS and runtime UI surface it contributes.

The package manifest implementation is split by responsibility: module-kind declarations, module contribution declarations, package declarations, constructors, and the built-in package catalog live in separate files. `package_manifest/mod.rs` remains a structural re-export boundary only.

Plugin component ids are full dotted names such as `weather.Component.CloudLayer`. They are intentionally not collapsed into a special `World` branch. `World` stores dynamic plugin component payloads per entity, exposes them through `dynamic_component(...)`, and routes property reads/writes through the same `ComponentPropertyPath` surface used by built-in components. Built-in components keep their existing high-performance maps; plugin components sit beside them for serialization, inspector property enumeration, and editor mutation.

The current dynamic component payload is `serde_json::Value` so VM and native plugin hosts can feed the same storage through stable data handles. Scalar writes use the engine `ScenePropertyValue` conversion path, which keeps editor field edits and serialized dynamic component data aligned.

## Runtime Validation Notes

The focused runtime coverage now checks three contracts:

- registry collection and duplicate diagnostics for managers, modules, render features, components, and UI components
- package manifest round-trip of runtime/editor contributions plus component declarations
- SourceTemplate generation of linked runtime crate `plugin_registration()` calls and `EntryRunner::bootstrap_with_runtime_plugin_registrations(...)`
- app startup acceptance of a required external runtime plugin when a linked registration report contributes the requested module
- app startup rejection of a linked registration report whose target modes do not match the requested runtime target
- plugin workspace runtime-package behavior tests for physics, animation, sound, net, navigation, particles, texture, virtual geometry, and hybrid GI, including render feature pass/capability metadata for the advanced render plugins
- descriptor-derived package manifests for runtime plugins preserve the target modes declared in each package `plugin.toml`
- SourceTemplate export generation after `RuntimePluginCatalog::builtin().complete_project_manifest(...)` preserves builtin catalog target modes in the generated `src/zircon_plugins.rs` project plugin selections
- renderer feature asset behavior for descriptor-owned plugin features, duplicate plugin feature names, and capability-driven advanced runtime flags
- default forward/deferred pipeline assets excluding pluginized advanced VG/GI built-in feature slots at construction time, with legacy built-in behavior isolated to explicit test fixtures
- runtime submission and scene renderer runtime flags ignoring legacy VG/GI built-in identity unless the compiled descriptor declares the matching capability requirement
- render pass executor registry coverage excluding VG/GI from the base built-in executor list and accepting those executor ids only through linked plugin render descriptors
- pipeline asset registration/reload rejection of plugin executor ids when the asset declares them behind quality or capability gates but the framework was not constructed with the matching linked render descriptor, plus acceptance when the descriptor is linked
- app startup render submission with a linked `virtual_geometry` registration report, proving contributed render feature descriptors rebuild the default pipeline and replace the old core virtual-geometry pass list before execution stats are recorded
- app startup render submission without linked advanced render plugins, proving a VG/GI quality request no longer executes old core advanced passes from the base pipeline
- app startup render submission with an explicitly registered legacy built-in pipeline fixture, proving VG/GI quality flags only open capability-gated plugin descriptors and do not reopen old `BuiltinRenderFeature` pass lists
- dynamic plugin component attach, world serialization round-trip, property read, and property write through existing scene property paths
- plugin unload guard behavior: `World::ensure_plugin_components_can_unload(plugin_id)` rejects unload while any entity still owns a dynamic component whose id starts with that plugin namespace, and succeeds after those components are removed
- target-client module ordering keeps GraphicsBase inside the minimal runtime core and appends UI as the default client plugin, matching the compile-time feature split
