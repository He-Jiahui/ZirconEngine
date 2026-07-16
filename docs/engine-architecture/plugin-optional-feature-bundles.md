---
related_code:
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_bundle_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_dependency.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/capability_status.rs
  - zircon_runtime/src/plugin/plugin_maturity.rs
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/groups.rs
  - zircon_runtime/src/plugin/runtime_profile/availability.rs
  - zircon_runtime/src/plugin/runtime_profile/availability_report.rs
  - zircon_runtime/src/plugin/runtime_profile/defaults.rs
  - zircon_runtime/src/plugin/runtime_profile/descriptor.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_feature_selection.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/native.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/status.rs
  - zircon_runtime/src/plugin/extension_registry/validation.rs
  - zircon_runtime/src/plugin/extension_registry/validation/plugin_event_catalog.rs
  - zircon_runtime/src/plugin/extension_registry/validation/plugin_option.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/native.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/project_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/status.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/diagnostic.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions/asset_scene.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions/component.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions/plugin_metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge/diagnostic.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/status.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities/base.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities/declaration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion/owner_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/definition.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/definition_map.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/key.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/lookup.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report/block.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report/dependency_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report/diagnostic.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_blocking.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_blocking/cycle.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution/availability.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution/availability/outcome.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection/active.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection/partition.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection/pending.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_status.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_status/dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/features.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/features/context.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/enabled_packages.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/feature_diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/feature_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/runtime_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/completion.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/selection_defaults/catalog_selections.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/lookup.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/selection_defaults.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/selection_defaults/hydration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/reports.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions/prepare.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions/providers.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_extensions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/conflict.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/registration_match.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/status.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/categories.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/model.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/media.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/pipeline.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/model.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/media.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/pipeline.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/content.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/services.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/content.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/language.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/language/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/language/classification.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/optional_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/net_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/net_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/net_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/feature_selection.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile.rs
  - zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_feature_selection_update_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/package_projection/native_project_selection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/features/timeline_animation_track/runtime/Cargo.toml
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/Cargo.toml
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/package_manifest_declarations.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
implementation_files:
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_bundle_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_dependency.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/plugin/runtime_profile/availability.rs
  - zircon_runtime/src/plugin/runtime_profile/availability_report.rs
  - zircon_runtime/src/plugin/runtime_profile/defaults.rs
  - zircon_runtime/src/plugin/runtime_profile/descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/native.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/status.rs
  - zircon_runtime/src/plugin/extension_registry/validation.rs
  - zircon_runtime/src/plugin/extension_registry/validation/plugin_event_catalog.rs
  - zircon_runtime/src/plugin/extension_registry/validation/plugin_option.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/native.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/project_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report/status.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/diagnostic.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions/asset_scene.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions/component.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/descriptor_contributions/plugin_metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge/diagnostic.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_merge/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/status.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities/base.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities/declaration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_capabilities/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion/owner_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection/package.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/definition.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/definition_map.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/key.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/lookup.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report/block.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report/dependency_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report/diagnostic.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_blocking.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_blocking/cycle.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution/availability.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution/availability/outcome.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection/active.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection/partition.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_selection/pending.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_status.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_status/dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/features.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/features/context.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/enabled_packages.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/feature_diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/feature_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/runtime_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/completion.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/lookup.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/selection_defaults.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/selection_defaults/catalog_selections.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/selection_defaults/hydration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/reports.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions/prepare.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/render_contributions/providers.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_extensions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/conflict.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/runtime_feature_definitions/registration_match.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/status.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/categories.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/model.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/media.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/pipeline.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/model.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/media.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/importer_classification/pipeline.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/content.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/services.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/content.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/language.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/language/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/language/classification.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/optional_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/net_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/net_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/net_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/Cargo.toml
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/Cargo.toml
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - user: 2026-05-03 optional feature bundle validation
  - user: 2026-05-08 实现 ZirconEngine Bevy 级插件完成度里程碑计划
  - user: 2026-05-16 continue Bevy-style runtime profile plugin group selection completion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/package_manifest_declarations.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/tests/mod.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_app --locked --offline --jobs 1 --features "ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1
doc_type: module-detail
---

# Plugin Optional Feature Bundles

Optional feature bundles model cross-plugin features as children of one owner plugin. The feature is shown and selected under the owner plugin, but its runtime/editor implementation can live in its own crate under `zircon_plugins/<owner>/features/<feature_slug>/...`.

The independent-provider model keeps the owner-facing selection model while letting a separate package provide the implementation. This is the only supported external-provider architecture; there is no owner-key fallback or legacy compatibility lookup. A package with `package_kind = "feature_extension"` declares its exported bundles in `feature_extensions`; those bundles are projected under `owner_plugin_id` and recorded in the project selection with `provider_package_id = <feature package id>`.

## Rules

- A feature has exactly one `owner_plugin_id`.
- The owner must also appear in `dependencies` with `primary = true`, and no secondary dependency may be marked primary.
- Dependencies are all-of in v1. They refer to plugin ids and public capabilities, not crate names.
- A feature is available only when the owner plugin is enabled, every dependency plugin is enabled for the target mode, every required capability is present from enabled plugins or earlier enabled features, and the feature target mode supports the export/runtime target.
- A feature-extension package is not registered as a base runtime plugin. It is a provider package for one or more `feature_extensions`, and each selected external feature requires that provider package to be enabled for the target mode.
- `ProjectPluginFeatureSelection.provider_package_id` is omitted for owner-embedded features and set only when an independent provider package owns the runtime/editor link unit.
- Optional blocked features become warnings. Required blocked features become fatal runtime/export diagnostics.
- Runtime code still declares real service dependencies with `DependencySpec`; plugin enablement is only an availability gate.
- For first-party owner-embedded features, the built-in runtime catalog mirrors the static `zircon_plugins/<owner>/plugin.toml` `optional_features` rows exactly. The catalog rows are runtime-facing projection data, not an independent source of feature dependency, module, capability, packaging, or default-enable truth.

## Runtime Flow

`PluginPackageManifest.optional_features` carries owner-embedded feature bundles. `PluginPackageManifest.feature_extensions` carries independent feature-package bundles, with `package_kind = FeatureExtension` marking pure provider packages. `RuntimePluginCatalog::complete_project_manifest` mirrors both declaration forms into `ProjectPluginSelection.features` as disabled selections by default, preserving package declaration order so project manifests do not drift across runs. External projections keep the owner row but fill `provider_package_id`. Completion also materializes a disabled provider `ProjectPluginSelection` when the external provider is not already present, deriving target modes and runtime/editor crate identity from the feature modules. Dependency enablement can therefore activate the provider through the same project-selection contract instead of inventing an Editor-only registration path.

Feature-definition identity follows the same hard-cut rule. `package_feature_definitions(...)` preserves an explicit `PluginFeatureBundleManifest.provider_package_id` for ordinary owner packages and defaults to the owner only when the field is absent. Consequently `feature_id@provider_package_id` is constructed once and used consistently by catalog lookup, project completion, status, and enablement; unknown providers remain structured failures rather than falling back to an owner-key alias.

The 2026-07-11 Editor M1 regression closes this boundary with a real Sound feature whose owner is `sound` and provider is `sound_timeline_animation_track`. The fully qualified Editor test `editor_manager_plugin_status_lists_owner_optional_feature_dependencies` passed 1/1 in 0.12s and asserts that `sound`, `animation`, and the external provider selection are all enabled. No duplicate catalog definition, alias key, or legacy provider fallback was added.

`PluginFeatureBundleManifest` carries dependency, capability, packaging, and explicit runtime/editor module rows for the feature bundle itself. Feature-bundle fixtures therefore declare executable link units through `with_runtime_module(PluginModuleManifest::runtime(...))` and `with_editor_module(PluginModuleManifest::editor(...))`; package-level crate shortcut builders remain on package/project manifest surfaces instead of being duplicated on feature bundles.

Fresh focused validation on 2026-06-03 came from the M6 root workspace test gate. `cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-asset-m6-root-0603-final --color never` stopped before running tests because `runtime_plugin_descriptor.rs` still used stale feature-bundle crate shortcut builders. The fixture now relies on explicit runtime/editor module rows, matching the feature-bundle API above; `cargo fmt -p zircon_runtime -- zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs`, `git diff --check -- zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs`, and `cargo test -p zircon_runtime --lib runtime_plugin_descriptor_projects_default_packaging_to_project_selection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-m6-root-0603-final --message-format short --color never -- --test-threads=1` passed. The full root workspace test still needs rerun after active neighboring Cargo lanes settle.

`RuntimePluginCatalog::feature_dependency_report` then evaluates enabled feature selections, resolves feature-provided capabilities in dependency order, and reports missing plugins, missing provider packages, missing capabilities, target mismatch, duplicate ids, and cycles. Feature definitions are keyed by feature id plus provider package id internally so a runtime registration from an independent provider can match the projected owner feature without colliding with owner-embedded feature registration.

Base plugin registration reports are merged first. Available feature registration reports are merged afterward, so feature modules/managers/components/render extensions can depend on services supplied by their owner and secondary plugins.

The catalog accepts the normal two-part representation where a package manifest declares a feature and the feature crate registers the same feature id at runtime. That pair is treated as one definition as long as the owner, dependencies, modules, capabilities, default packaging, and default enablement match. Two package declarations with the same feature id, two runtime feature registrations with the same feature id, or a package/runtime pair whose core contract differs remain structured diagnostics.

Cycle diagnostics only apply to enabled unresolved features that wait on capabilities declared by each other. A disabled feature that could provide a missing capability is reported as a missing capability instead, so the editor can guide the user to enable dependencies without presenting a false cycle.

`RuntimeExtensionCatalogReport` preserves all runtime registration notes in `diagnostics` and mirrors only hard failures into `fatal_diagnostics`. Blocked optional features are therefore visible to hosts without making the runtime extension report fail. Blocked required features, duplicate/ambiguous feature definitions, and registry merge errors are fatal; `is_success()` checks the fatal list rather than requiring all informational diagnostics to be empty.

Native plugin package manifests participate in the same model. The native loader preserves `optional_features` and `feature_extensions` while merging discovered manifest, descriptor, runtime entry, and editor entry package metadata, then projects runtime-capable native packages and their runtime-capable optional features into registration reports so dependency status can be evaluated with the built-in catalog plus discovered native packages. Discovery can use a runtime/editor module declared inside `feature_extensions` for pure feature-extension packages, and `runtime_plugin_registration_reports()` deliberately skips `package_kind = FeatureExtension` packages so they only contribute feature registrations.

## Editor Status Flow

The Plugin Manager status report projects `optional_features` under the owner plugin. Each feature row records whether the feature is enabled, whether its all-of dependency set is currently available for the editor host target, which runtime/editor crates would be linked, which capabilities it provides, and a dependency checklist with plugin/capability readiness for each required dependency.

The pane payload keeps the summary nested under the owner plugin so UI details can show missing plugins and missing capabilities without pretending that a checked plugin toggle replaces runtime `DependencySpec` service declarations.

Feature actions are explicit. Enabling a feature first asks the runtime catalog whether the candidate feature is available for `editor_host`; if any plugin, capability, owner-primary, or target-mode gate is missing, the action is blocked with a structured diagnostic. The dependency action updates only the dependency selections: it enables required dependency plugins and, when a dependency capability is provided by exactly one optional feature under that dependency plugin, enables that provider feature too. Provider features are resolved recursively so combinations such as `rendering.vfx_graph -> rendering.shader_graph` can be prepared in one dependency action, while cycles and multiple providers remain diagnostics. It does not silently enable the target feature; the user still confirms the feature after dependencies are ready.

Native-aware status reports use the same projection helpers as built-in plugin status. A native plugin discovered only through `plugin.toml` still shows its optional feature rows, dependency checklist, default feature crates, packaging, and target compatibility, while load-state diagnostics such as a missing dynamic library remain attached to the native plugin row.

Native-aware project completion now preserves those optional feature selections in the project manifest projection, including feature crate names, target modes, and native-default packaging. Feature enablement actions use the same built-in plus native catalog as native-aware status, so a feature declared only in `zircon_plugins/<plugin>/plugin.toml` can prepare dependencies and then enable without falling back to the built-in-only catalog.

## Profile and Maturity Flow

M1 adds explicit maturity and capability-status metadata beside the existing optional-feature model. `RuntimePluginDescriptor` and `PluginPackageManifest` now carry `PluginMaturity`, while `PluginPackageManifest.capability_statuses` records the status of each public capability or feature capability. These fields are metadata gates only; they do not replace `DependencySpec` or `RuntimeExtensionRegistry` contributions.

`RuntimeProfileDescriptor` groups default and optional runtime plugin selections for `minimal`, `client_2d`, `client_3d`, `editor`, `dev`, and `server`. The runtime profile boundary is folder-backed: `runtime_profile.rs` only publishes the API surface, `descriptor.rs` owns profile declarations and deterministic `ProjectPluginManifest` projection, `defaults.rs` owns the built-in profile rows, `availability_report.rs` owns the structured report buckets and diagnostic lines, and `availability.rs` owns provider/link/native-dynamic availability evaluation. The resulting `RuntimePluginAvailabilityReport` separates available, linked, externalized, stub, target-blocked, maturity-blocked, and missing-required plugin states without mixing those policy checks into the declaration structs.

Stable/default-style profiles reject required `Externalized`, `Stub`, and below-minimum maturity plugins. Optional advanced plugins such as particles, virtual geometry, hybrid GI, and physics use the same report gates but populate warning buckets without blocking `missing_required`.

The M2 provider-aware path distinguishes descriptor maturity from actual provider availability. Linked first-party registration reports satisfy `linked`; native dynamic registration reports satisfy `native_dynamic`; required profile plugins with no linked/native provider now remain `externalized_missing` even when their catalog descriptor is mature enough for the profile. Provider reports do not bypass target, stub, or minimum-maturity gates; a linked provider only satisfies availability after descriptor metadata proves the plugin is acceptable for the selected profile.

The default app-side linked provider is feature-gated in `zircon_app`. `EntryConfig::for_runtime_profile()` chooses the app entry mode and projected profile manifest, then `first_party_runtime_plugin_registrations_for_config()` converts enabled selections into linked `RuntimePluginRegistrationReport` values from compiled first-party plugin crates. This gives profile bootstrap the same provider shape as generated export hosts while preserving the runtime boundary: optional-feature dependency checks and module registration still consume reports, not concrete plugin crate types.

`RuntimePluginRegistrationReport` keeps the public base-plugin report shape, but its implementation is folder-backed: linked provider report construction lives in `registration_report/plugin.rs`, native package report construction in `registration_report/native.rs`, and status reporting in `registration_report/status.rs`.

The app provider path also keeps profile-owned platform/render configuration authoritative across activation. Bootstrap stores `PlatformConfig` and `RenderProfileBundle` before module activation and writes them again afterward, so module defaults cannot overwrite a selected headless render bundle or the minimal profile's disabled platform state.

## Export Flow

`ExportBuildPlan` links only active feature runtime crates. Owner-embedded features link from `zircon_plugins/<owner>/features/<feature_slug>/runtime`; external provider features link from `zircon_plugins/<provider_package_id>/runtime` when selected as `LibraryEmbed`. Generated source exports both:

- `runtime_plugin_registrations()` for base plugins.
- `runtime_plugin_feature_registrations()` for available optional features.

Generated `main.rs` calls `EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations`, preserving the base-plugin-first, feature-second ordering at runtime.

Blocked optional features remain in `diagnostics` only and are not linked. Blocked required features and structural feature-definition diagnostics are copied into both `diagnostics` and `fatal_diagnostics`, and materialization/editor export reports preserve the fatal list so export hosts can block packaging or surface a hard failure without parsing diagnostic strings.

Native dynamic exports now merge both `runtime_plugin_registration_reports()` and `runtime_plugin_feature_registration_reports()` from the native loader. An owner-embedded feature selected as `NativeDynamic` still travels through its owner plugin's native dynamic package; if the export profile lacks `NativeDynamic`, or the owner plugin is not selected as `NativeDynamic`, export emits a structured diagnostic and treats it as fatal for required features. An external feature selected as `NativeDynamic` exports its `provider_package_id` package independently, so the owner plugin can remain linked or source-templated while the feature provider travels as its own native package.

`RuntimePluginFeatureRegistrationReport` stays as the public report surface, but the implementation is folder-backed: linked trait-feature registration lives in `feature_registration_report/feature.rs`, native manifest registration in `feature_registration_report/native.rs`, provider override state in `feature_registration_report/provider.rs`, and project-selection projection in `feature_registration_report/project_selection.rs`.

Built-in runtime plugin rows are folder-backed too: `builtin_catalog/core_rows.rs` now only chains core row groups, `core_rows/runtime.rs` only chains runtime foundation row subgroups, `core_rows/runtime/services.rs` owns physics, sound, texture, and net service rows, `core_rows/runtime/systems.rs` owns navigation, particles, and animation system rows, and `core_rows/content.rs` owns terrain, tilemap, and prefab-tool rows while preserving descriptor order before asset, render, and language row groups.

Built-in core descriptor classification now mirrors that row split: `builtin_catalog/core_classification.rs` only routes classification, `core_classification/runtime.rs` only chains runtime foundation classification subgroups, `core_classification/runtime/services.rs` owns physics, sound, texture, and net maturity/capability-status metadata, `core_classification/runtime/systems.rs` owns navigation, particles, and animation metadata, and `core_classification/content.rs` owns terrain, tilemap, and prefab-tool classification metadata.

Built-in asset importer rows follow the same ownership rule: `builtin_catalog/asset_rows.rs` now only chains model, media, and pipeline importer row groups, `asset_rows/model.rs` owns glTF and OBJ importers, `asset_rows/media.rs` owns texture and audio importers, and `asset_rows/pipeline.rs` owns WGSL shader and UI document importers while preserving descriptor order before render and language row groups.

Built-in importer classification now mirrors the asset row split too: `builtin_catalog/importer_classification.rs` only routes importer descriptor classification, `importer_classification/model.rs` owns glTF and OBJ maturity/capability-status metadata, `importer_classification/media.rs` owns texture and audio metadata while preserving the texture image capability override, and `importer_classification/pipeline.rs` owns WGSL shader and UI document importer metadata. Built-in importer primary-capability mapping now lives under `importer_classification/capabilities.rs`; shared `capability_status.rs` only constructs generic capability status rows.

Built-in language catalog metadata is folder-backed under one ZrVM language owner: `builtin_catalog/language.rs` only exposes the language row and classification entry points, `language/rows.rs` owns the ZrVM package row, and `language/classification.rs` owns the experimental maturity plus primary and backend capability-status metadata.

Built-in descriptor augmentation is split by metadata kind: `builtin_catalog/augmentation.rs` now keeps only the category-then-capability orchestration, `augmentation/categories.rs` owns runtime/authoring/rendering/asset-importer category assignment, and `augmentation/capabilities.rs` owns the extra package capabilities for animation, ZrVM language, physics, and asset importers.

Built-in Net optional features are folder-backed too: `builtin_catalog/net_features.rs` now dispatches row data to manifest construction, `net_features/rows.rs` owns HTTP, WebSocket, RPC, replication, reliable UDP, and content-download rows, and `net_features/manifest.rs` owns the shared Net feature manifest builder.

Built-in Particles optional features follow the same row boundary: `builtin_catalog/particles_features.rs` now only dispatches ordered row data, `particles_features/rows.rs` owns the physics, animation-control, and GPU-simulation rows plus their required dependency rows, and `particles_features/manifest.rs` owns shared Particles feature manifest construction.

Built-in Sound optional features now use that same boundary while preserving runtime/editor feature modules: `builtin_catalog/sound_features.rs` dispatches ordered rows through `attach_sound_features`, `sound_features/rows.rs` owns the timeline-animation-track and ray-traced-convolution-reverb feature data, dependency rows, target modes, and provider crate names, and `sound_features/manifest.rs` owns the shared manifest builder.

Built-in Rendering optional features are folder-backed too without touching render runtime execution: `builtin_catalog/rendering_features.rs` only folds ordered feature rows into descriptors, `rendering_features/rows.rs` owns the nine Rendering feature rows, default-enable flags, and VFX Graph dependency rows, and `rendering_features/manifest.rs` derives the runtime/editor capability and crate metadata from those rows.

`RuntimePluginCatalog` stays as the public aggregation type, but catalog behavior is split by method family: `runtime_plugin_catalog/registration.rs` owns catalog construction and report ingestion, `access.rs` owns package/diagnostic accessors, `project.rs` owns project manifest completion and package selection lookup, `feature_dependencies.rs` owns the public optional-feature dependency report entry point, `runtime_extensions.rs` owns runtime extension report projection, and `status.rs` owns the success predicate. Feature-definition records are also folder-backed: `feature_definitions/definition.rs` owns the record, `definition_map.rs` owns the keyed collection, `lookup.rs` owns owner/provider selection matching, and `key.rs` owns provider key formatting. Feature definition collection now keeps package-declared rows in `feature_definition_collection/package.rs`, while runtime registration merge stays under `runtime_feature_definitions/`. The lower dependency-resolution and extension-merge modules remain separate siblings under the same owner folder. Runtime extension contribution fan-in is folder-backed as well: `contributions.rs` only wires the contribution owner, `contributions/extension.rs` owns registry merge orchestration, and `contributions/diagnostic.rs` owns registry-result diagnostic projection reused by descriptor and render contribution groups. Descriptor contribution groups are split into `descriptor_contributions/component.rs`, `plugin_metadata.rs`, and `asset_scene.rs`, keeping component/UI, option/event catalog, and asset/scene-hook registry loops separate while preserving the same optional-feature extension report path. Render contribution groups are split into `render_contributions/feature.rs`, `prepare.rs`, and `providers.rs`, keeping render feature/executor, runtime prepare, and advanced runtime provider rows separate before optional-feature extension reports consume the merged registry.

Runtime feature-definition merge is folder-backed too: `runtime_feature_definitions/merge.rs` owns runtime feature definition merge orchestration, `registration.rs` owns single runtime feature registration duplicate detection and definition insertion, `conflict.rs` owns package-declaration conflict projection, and `registration_match.rs` owns package-manifest registration equality checks.

Feature completion is folder-backed too: `feature_completion.rs` owns catalog feature-selection completion orchestration, and `feature_completion/owner_selection.rs` owns owner feature row hydration.

For catalogs that do not yet know an external feature package at source-generation time, export treats an enabled feature selection with `provider_package_id` and an enabled provider package selection as a deferred external provider. It links or packages that provider and suppresses only the "unknown feature" diagnostic for that deferred case; ordinary owner-embedded unknown features still report normally.

Feature dependency report DTOs are folder-backed as well: `feature_report/dependency_report.rs` owns the report row, `block.rs` owns blocked-feature data, and `diagnostic.rs` owns diagnostic formatting.

Feature capability helpers are folder-backed too: `feature_capabilities/base.rs` owns base runtime package capability collection, `feature.rs` owns feature-provided capability projection, and `declaration.rs` owns feature capability declaration checks.

Feature resolution is folder-backed as well: `feature_resolution.rs` owns pending-feature orchestration, `feature_resolution/availability.rs` owns the available-feature fixed-point loop, and `feature_resolution/availability/outcome.rs` owns available/blocked/waiting pending-feature status projection before unresolved waits are projected by the blocking module.

Feature blocking is folder-backed too: `feature_blocking.rs` owns final blocked-feature report projection, and `feature_blocking/cycle.rs` owns unresolved feature id collection plus feature-capability cycle marking.

Feature status is folder-backed too: `feature_status.rs` owns owner/provider/target status orchestration, and `feature_status/dependencies.rs` owns dependency-row missing-plugin and missing-capability accumulation.

Feature dependency report orchestration is folder-backed too: `features.rs` owns dependency report flow, while `features/context.rs` owns plugin-selection lookup, enabled package set, base capability seed, and initial dependency-report construction.

Project manifest support is folder-backed too: `project_manifest/completion.rs` owns catalog manifest construction and completion orchestration, `project_manifest/selection_defaults.rs` owns defaulting orchestration, `selection_defaults/catalog_selections.rs` owns missing package selection insertion, `selection_defaults/hydration.rs` owns runtime/editor crate and target-mode default hydration, and `project_manifest/lookup.rs` owns package selection lookup.

Extension merging is also folder-backed: `extension_merge/runtime.rs` owns runtime package merge, `feature.rs` owns feature extension merge, and `diagnostic.rs` owns fatal diagnostic fan-out.

Extension reports are folder-backed too: `extension_report/report.rs` owns `RuntimeExtensionCatalogReport`, `status.rs` owns report success helpers, and `runtime.rs` owns full-catalog runtime extension report assembly.

Project extension reports are folder-backed as well: `project_extension_report/enabled_packages.rs` owns project-enabled package id projection, `runtime_merge.rs` owns enabled runtime package extension merging, `feature_diagnostics.rs` owns feature dependency diagnostic projection, and `feature_merge.rs` owns available feature extension merge.

Catalog registration is folder-backed: `registration/constructors.rs` owns catalog constructors, `reports.rs` owns report ingestion, and `plugin.rs` owns direct plugin and feature registration mutation.

Feature selection is folder-backed: `feature_selection/active.rs` owns enabled feature enumeration, `pending.rs` owns pending feature-selection rows, and `partition.rs` owns catalog-backed pending versus unknown-feature partitioning.

## Current Examples

- `sound.timeline_animation_track`
  - owner: `sound`
  - dependencies: `sound/runtime.plugin.sound`, `animation/runtime.feature.animation.timeline_event_track`
  - provides: `runtime.feature.sound.timeline_animation_track`
  - runtime crate: `zircon_plugin_sound_timeline_animation_runtime`
  - editor capability: `editor.feature.sound.timeline_animation_track`
  - editor crate: `zircon_plugin_sound_timeline_animation_editor`

- `sound.ray_traced_convolution_reverb`
  - owner: `sound`
  - dependencies: `sound/runtime.plugin.sound`, `physics/runtime.plugin.physics`, `physics/runtime.capability.physics.raycast`
  - provides: `runtime.feature.sound.ray_traced_convolution_reverb`
  - runtime crate: `zircon_plugin_sound_ray_traced_convolution_runtime`
  - editor capability: `editor.feature.sound.ray_traced_convolution_reverb`
  - editor crate: `zircon_plugin_sound_ray_traced_convolution_editor`

- Rendering owner-embedded optional features
  - owner: `rendering`
  - feature ids: `rendering.post_process`, `rendering.ssao`, `rendering.contact_shadow`, `rendering.decals`, `rendering.reflection_probes`, `rendering.baked_lighting`, `rendering.ray_tracing_policy`, `rendering.shader_graph`, `rendering.vfx_graph`
  - primary dependency: `rendering/runtime.plugin.rendering`
  - extra dependency: `rendering.vfx_graph` also requires `particles/runtime.plugin.particles` and `rendering/runtime.feature.rendering.shader_graph`
  - provides: `runtime.feature.rendering.<feature>`
  - runtime crate: `zircon_plugin_rendering_<feature>_runtime`
  - editor capability: `editor.feature.rendering.<feature>`
  - editor crate: `zircon_plugin_rendering_<feature>_editor`

- `net.content_download`
  - owner: `net`
  - primary dependency: `net/runtime.plugin.net`
  - required feature dependency: `net/runtime.feature.net.http`
  - provides: `runtime.feature.net.cdn_download`
  - runtime crate: `zircon_plugin_net_content_download_runtime`
  - transport rule: content downloads use the shared HTTP-capable `NetManager` rather than declaring an independent client stack
  - catalog rule: static `plugin.toml`, linked Net package manifest, provider manifest, and built-in runtime catalog all expose the same dependency contract
  - metadata rule: static `plugin.toml`, linked Net package manifest, and built-in catalog classify the Net owner package as category `runtime`, maturity `beta`, and `runtime.plugin.net` status `partial`

- `sound_timeline_animation_track` as an independent provider package
  - package kind: `feature_extension`
  - declares: `feature_extensions = ["sound.timeline_animation_track"]`
  - projected owner row: `sound.features["sound.timeline_animation_track"]`
  - provider gate: project must also enable plugin/package `sound_timeline_animation_track`
  - embedded path: `zircon_plugins/sound_timeline_animation_track/runtime`

## Validation

Added coverage for manifest roundtrip, project manifest nested feature selections, deterministic catalog completion, dependency availability/blocking, target mismatch diagnostics, disabled provider versus feature-cycle diagnostics, package/runtime declaration default mismatches, runtime blocked optional-vs-required fatal semantics, export linking/diagnostics/fatal diagnostics, native feature registration projection, runtime extension merge ordering, editor Plugin Manager status projection, recursive dependency enablement, native manifest optional-feature projection, native-aware project completion, editor feature/dependency action projection, and static manifest versus built-in catalog optional-feature parity.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `extension_report.rs` now delegates its report declaration, status helpers, and full-catalog runtime extension report assembly to `extension_report/report.rs`, `status.rs`, and `runtime.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 30.01s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `feature_resolution.rs` now delegates available-feature fixed-point resolution to `feature_resolution/availability.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 24.24s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `feature_resolution/availability.rs` now keeps the available-feature fixed-point loop while `feature_resolution/availability/outcome.rs` owns available/blocked/waiting pending-feature status projection; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 2.01s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `feature_blocking.rs` now keeps final blocked-feature report projection while `feature_blocking/cycle.rs` owns unresolved feature id collection and feature-capability cycle marking; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 20.57s with 17 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/core_rows.rs` now keeps core built-in row-group ordering while `core_rows/runtime.rs` owns runtime foundation package rows and `core_rows/content.rs` owns content/tool package rows; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 2.10s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `core_rows/runtime.rs` now keeps only runtime foundation row-subgroup orchestration, `core_rows/runtime/services.rs` owns physics, sound, texture, and net rows, and `core_rows/runtime/systems.rs` owns navigation, particles, and animation rows while preserving the original core descriptor order. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 21.51s with 11 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/net_features.rs` now dispatches Net optional-feature rows to `net_features/manifest.rs`, while `net_features/rows.rs` owns the ordered Net feature catalog rows and content-download HTTP dependency row; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 26.06s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `feature_status.rs` now delegates feature dependency-row missing-plugin and missing-capability accumulation to `feature_status/dependencies.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 28.71s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `features.rs` now delegates feature dependency report context construction to `features/context.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 20.56s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `project_manifest/selection_defaults.rs` now delegates missing catalog selection insertion to `selection_defaults/catalog_selections.rs` and runtime/editor crate plus target-mode default hydration to `selection_defaults/hydration.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 23.52s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `feature_capabilities.rs` now delegates base runtime package capability collection, feature-provided capability projection, and feature declaration checks to `feature_capabilities/base.rs`, `feature.rs`, and `declaration.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 24.20s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `runtime_feature_definitions.rs` now delegates runtime feature definition merge and package-manifest registration equality checks to `runtime_feature_definitions/merge.rs` and `registration_match.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 20.09s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `runtime_feature_definitions/merge.rs` now delegates single runtime feature registration duplicate detection, package-declaration conflict projection, and definition insertion to `runtime_feature_definitions/registration.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 47.93s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `runtime_feature_definitions/registration.rs` now delegates package-declaration conflict projection to `runtime_feature_definitions/conflict.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 22.97s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `feature_completion.rs` now delegates owner feature row hydration to `feature_completion/owner_selection.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 22.98s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `contributions.rs` now delegates runtime extension registry fan-in to `contributions/extension.rs` and registry-result diagnostic projection to `contributions/diagnostic.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 27.30s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `descriptor_contributions.rs` now delegates component/UI descriptor rows to `descriptor_contributions/component.rs`, option/event catalog rows to `plugin_metadata.rs`, and asset importer plus scene-hook rows to `asset_scene.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 20.07s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `render_contributions.rs` now delegates render feature/pass executor rows to `render_contributions/feature.rs`, runtime prepare collectors to `prepare.rs`, and Virtual Geometry, Hybrid GI, and Solari runtime providers to `providers.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 27.45s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `project_manifest/completion.rs` now delegates missing package selection insertion and runtime/editor crate plus target-mode default hydration to `project_manifest/selection_defaults.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 22.18s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `feature_definition_collection.rs` now delegates package-declared optional-feature and feature-extension definition insertion to `feature_definition_collection/package.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 38.37s with 10 existing warnings.

Fresh focused validation attempt on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `project_extension_report.rs` now delegates enabled runtime package extension merging to `project_extension_report/runtime_merge.rs`; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, and tracked-file `git diff --check -- ...` passed. The scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` attempt stopped in the active WGPU render-graph lane with `E0061` at `zircon_runtime/src/render_graph/builder.rs:175`, `:209`, and `:234` after `add_resource_access(...)` was changed to require a fourth argument; `.codex/sessions/20260602-0043-wgpu-render-main-chain.md` owns that active `render_graph` surface, so no Cargo acceptance pass is recorded for this plugin-only slice.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/particles_features.rs` now dispatches ordered Particles optional-feature row data to `particles_features/manifest.rs`, while `particles_features/rows.rs` owns the physics, animation-control, and GPU-simulation dependency rows; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 2m20s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/optional_features.rs` now dispatches Sound through `attach_sound_features`, `builtin_catalog/sound_features.rs` dispatches ordered Sound optional-feature rows to `sound_features/manifest.rs`, and `sound_features/rows.rs` owns timeline-animation-track plus ray-traced-convolution-reverb dependency/module metadata; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 25.96s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/rendering_features.rs` now dispatches ordered Rendering optional-feature rows to `rendering_features/manifest.rs`, while `rendering_features/rows.rs` owns the eight feature rows, default-enable flags, and VFX Graph dependency rows; focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 25.34s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/asset_rows.rs` now dispatches ordered asset importer rows through model, media, and pipeline child modules; `asset_rows/model.rs` owns glTF and OBJ rows, `asset_rows/media.rs` owns texture and audio rows, and `asset_rows/pipeline.rs` owns WGSL shader and UI document rows. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 20.32s with 10 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/core_classification.rs` now dispatches core descriptor classification through runtime foundation and content/tool child modules; `core_classification/runtime.rs` owns physics, sound, texture, net, navigation, particles, and animation classification metadata, while `core_classification/content.rs` owns terrain, tilemap, and prefab-tool classification metadata. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 20.17s with 11 existing warnings.

Fresh focused validation attempt on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `core_classification/runtime.rs` now keeps only runtime foundation classification orchestration, `core_classification/runtime/services.rs` owns physics, sound, texture, and net maturity/capability-status metadata, and `core_classification/runtime/systems.rs` owns navigation, particles, and animation metadata while preserving maturity rows, capability-status rows, Bevy references, notes, routing, and public catalog surface.
Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, and tracked-file `git diff --check -- ...` passed; the tracked-file diff check reported only expected LF-to-CRLF warnings for the two touched docs.
The fresh scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` attempt stopped in the active graphics/scene lane with 10 errors at `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs:294`, `:307-309`, `:335`, `:346`, `:350`, `:375`, and `:387-388`; `.codex/sessions/20260602-0043-wgpu-render-main-chain.md` owns that active WGPU/render surface, so no Cargo acceptance pass is recorded for this plugin-only slice.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/importer_classification.rs` now dispatches importer descriptor classification through model, media, and pipeline child modules; `importer_classification/model.rs` owns glTF and OBJ classification metadata, `importer_classification/media.rs` owns texture and audio metadata while preserving `texture_importer`'s explicit `runtime.asset.importer.texture.image` capability status, and `importer_classification/pipeline.rs` owns WGSL shader and UI document metadata. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 20.83s with 11 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/augmentation.rs` now dispatches descriptor augmentation through category and capability child modules; `augmentation/categories.rs` owns category assignment, while `augmentation/capabilities.rs` owns extra package capability attachment and preserves the previous category-then-capability order for asset importers. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 19.10s with 11 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/capability_status.rs` now only constructs generic `CapabilityStatusManifest` rows, while `importer_classification/capabilities.rs` owns glTF, OBJ, audio, WGSL shader, UI document, and fallback `_importer` primary-capability mapping for importer classification. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 2.35s with 11 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `builtin_catalog/language.rs` now owns the ZrVM language catalog boundary, `language/rows.rs` owns the ZrVM built-in row, and `language/classification.rs` owns the experimental maturity plus `runtime.plugin.zr_vm_language` and `runtime.script.backend.zr_vm_project` capability-status metadata. Focused rustfmt, old flat language-path scan, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 2.72s with 12 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `export_build_plan/from_project_manifest.rs` now keeps the export profile, catalog completion, linked runtime crate, native package, generated file, and availability assembly flow while `from_project_manifest/feature_selection.rs` owns project feature/provider selection helpers and `from_project_manifest/profile.rs` owns export-profile to runtime-profile inference. Behavior, diagnostics, generated SourceTemplate projection, native package projection, and public `ExportBuildPlan::from_project_manifest(...)` surface are unchanged. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 2.48s with 11 existing warnings; diff check reported only expected LF-to-CRLF warnings for the three touched docs and `from_project_manifest.rs`.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `extension_registry/validation.rs` now keeps non-metadata registry validation while `extension_registry/validation/plugin_option.rs` owns plugin option key/value/default/enum/required-capability validation and `extension_registry/validation/plugin_event_catalog.rs` owns event catalog namespace, event id, version, and payload-schema validation. Behavior and diagnostic text are unchanged. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 28.59s with 11 existing warnings.

Fresh focused validation on 2026-06-02 used `D:\cargo-targets\zircon-runtime-plugin-boundary`: `export_build_plan/platform_host_files.rs` now keeps host-kind dispatch, desktop/headless source-template entry generation, runtime library scaffolding, and shared name/escape helpers; `platform_host_files/mobile.rs` owns Android/iOS file lists and templates, while `platform_host_files/browser.rs` owns WebGPU/WASM browser file lists and templates. Generated file paths, purposes, contents, and public export-plan behavior are unchanged. Focused rustfmt, code migration-word scan, direct whitespace/conflict scans, tracked-file `git diff --check -- ...`, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never` passed in 2.23s with 11 existing warnings; diff check reported only expected LF-to-CRLF warnings for two docs and `platform_host_files.rs`.

Fresh focused validation on 2026-06-04 used `D:\cargo-targets\zircon-runtime-plugin-ecosystem-continuation`: `export_build_plan.rs` now has a Rendering owner-feature export regression proving that a selected `rendering` package links the base runtime crate plus the default-enabled `post_process`, `ssao`, `reflection_probes`, and `baked_lighting` feature runtime crates, emits their generated `plugin_feature_registration()` calls, and keeps opt-in Rendering feature crates out of the SourceTemplate until explicitly enabled. `cargo test -p zircon_runtime --lib source_template_links_rendering_default_owner_features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-ecosystem-continuation --message-format short --color never -- --test-threads=1 --nocapture` passed with 12 existing runtime warnings. A tracked-file `git diff --check -- zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs` check reported only the expected LF-to-CRLF warning.

2026-07-01 Runtime 15 test-owner maintenance: `Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split` / `runtime_15_runtime_plugin_catalog_features_dependency_report_tests_child_owner_split_static_passed_cargo_deferred` keeps optional-feature dependency-report tests folder-backed without changing `RuntimePluginCatalog` production behavior. `zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features.rs` now mounts `zircon_runtime/src/tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs`; the parent keeps catalog completion, external feature projection, runtime extension merge, and shared sound/animation fixtures, while the child owns optional dependency status, provider selection, invalid primary dependency, target mismatch, disabled-provider, and cycle diagnostics. `runtime_15_runtime_plugin_catalog_features_dependency_report_tests_are_child_owner` locks the mount, moved-test non-regression, 11-test total, and Runtime 15 800-line owner budget. Cargo remains deferred under active cargo/rustc lanes.

The Sound provider crates now also carry local feature-registration contracts for `sound.timeline_animation_track` and `sound.ray_traced_convolution_reverb`. These tests keep provider-owned runtime `feature_manifest()` output aligned with the owner/static Sound bundle rows for id, display name, owner, dependency set, provided runtime capability, default packaging, runtime module, and editor module metadata, including the editor capability rows projected by the editor host. The provider runtime crates depend on `zircon_runtime` with default features disabled because they only need the plugin/core metadata surface, not the full render/runtime stack. The ray-traced convolution provider display name is intentionally `Ray Traced Convolution Reverb`, matching the static Sound bundle and generated owner descriptor, and the editor feature crates reuse the provider `EDITOR_CAPABILITY` constants.

Fresh focused validation on 2026-05-31 used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-feature-editor-capability`: both Sound feature provider manifest tests passed after the editor-module capability rows were added, the Sound runtime `manifest` filter passed static optional feature parity, the new runtime catalog regression `builtin_sound_optional_features_declare_editor_capabilities` passed, both feature editor crates passed `cargo check`, and locked offline metadata passed for the Sound runtime plus both feature runtime/editor manifests.

Fresh focused validation on 2026-05-31 used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-rendering-feature-editor-capability`: the new red runtime catalog regression first failed because `rendering.post_process.editor` did not project `editor.feature.rendering.post_process`; after the fix, Rendering provider manifests passed `rendering_feature_manifests_declare_editor_capabilities`, the built-in catalog regression `builtin_rendering_optional_features_declare_editor_capabilities` passed, the static `rendering_plugin_toml_roundtrips_owner_features_and_modules` parity test passed, and all eight Rendering feature editor crates passed `cargo check`. A broad package-level `zircon_runtime` test attempt was intentionally not used as acceptance because the current dirty worktree has an unrelated `virtual_geometry_debug_snapshot_contract.rs` compile error for `ModelPrimitiveAsset { .. }` missing `mesh`.

Fresh focused validation on 2026-05-31 used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-net-content-download-manifest`: red tests first showed that the base Net runtime package manifest and static `zircon_plugins/net/plugin.toml` did not declare the `net.content_download` dependency on `runtime.feature.net.http`; after the fix, `net_plugin_manifest_advertises_layered_optional_features`, `net_plugin_toml_declares_content_download_http_dependency`, and the full `zircon_plugin_net_content_download_runtime` lib test set passed.

Fresh focused validation on 2026-05-31 used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-net-builtin-catalog`: red testing first showed that `RuntimePluginDescriptor::builtin_catalog()` had no Net optional feature rows; after adding the six owner feature rows, `builtin_net_catalog_declares_layered_optional_features`, `builtin_net_content_download_dependency_report_blocks_without_http_feature`, `net_plugin_toml_declares_content_download_http_dependency`, and `net_plugin_manifest_advertises_layered_optional_features` passed.

Fresh focused validation on 2026-05-31 used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-net-runtime-metadata`: red testing first showed that the linked Net runtime package still reported `Experimental` maturity and static `zircon_plugins/net/plugin.toml` still defaulted to `uncategorized`; after the fix, `net_plugin_manifest_advertises_layered_optional_features` and `net_plugin_toml_declares_content_download_http_dependency` passed with category `runtime`, maturity `beta`, and `runtime.plugin.net` status `partial`.

Fresh validation on 2026-05-03:

- `cargo fmt -p zircon_runtime -p zircon_editor`
- `cargo metadata --locked --no-deps --format-version 1`
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_runtime --lib native_manifest_merge_preserves_optional_feature_declarations --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib feature_status_rejects_secondary_primary_dependency --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib native_plugin_status_uses_manifest_when_library_is_missing --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib editor_manager_plugin_status_lists_owner_optional_feature_dependencies --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib editor_manager_feature_dependency_enablement_turns_on_unique_provider_features --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib shared_menu_pointer_layout --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib root_menu_popup_scroll_and_dismiss_flow_through_shared_pointer_bridge_in_real_host --locked --jobs 1 --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib runtime_plugin_catalog_reports_target_mismatch_for_optional_feature --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib runtime_plugin_catalog_reports_feature_capability_cycles --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_editor --lib native_plugin_status_uses_manifest_when_library_is_missing --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_editor --lib native_selection_preserves_optional_feature_defaults --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `cargo fmt -p zircon_runtime -p zircon_app -p zircon_editor`
- `cargo metadata --locked --no-deps --format-version 1`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib plugin_extensions --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never -- --test-threads=1`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib source_template_links_external_feature_provider_runtime_crates --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib native_load_report_projects_optional_features_as_runtime_feature_registrations --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo check -p zircon_app --lib --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_app --lib --locked --jobs 1 --target-dir target-codex-runtime-check --message-format short --color never`
- `git diff --check -- <optional-feature touched files>`

Fresh M2 app-provider validation on 2026-05-16 used `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target` because other active sessions were using the shared target directories:

- `cargo test -p zircon_app --locked --offline --jobs 1 --features "ui,first-party-runtime-plugins" entry_config_can_select_headless_render_profile_bundle -- --nocapture --test-threads=1` passed: 1 test, 0 failures.
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1` passed: 15 tests, 0 failures.
- `cargo test -p zircon_app --locked --offline --jobs 1 profile_bootstrap -- --nocapture --test-threads=1` passed: 13 tests, 0 failures.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" profile_bootstrap --message-format short -- --nocapture --test-threads=1` passed on Windows: 18 tests, 0 failures.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed on Windows: 1 test, 0 failures. The root lockfile keeps Slint/`zircon_hub`'s `accesskit_windows v0.30.0` on `windows 0.61.3`, but aligns `gpu-allocator v0.28.0` with `wgpu-hal v29.0.3` on `windows 0.62.2` so their D3D12 types match.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/opencode/zircon-profile-provider-target cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed in WSL/Linux: 1 test, 0 failures.

Workspace-wide `cargo build --workspace` / `cargo test --workspace` was not run in this session because the checkout is under active multi-session churn and this milestone used targeted package validation for the optional-feature surfaces.
