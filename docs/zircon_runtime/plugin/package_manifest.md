---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/package_manifest/mod.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_interface_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_module_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/extension_registry/validation.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/construction.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/fluent.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/owner.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/primary_count.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/row/capability.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/row/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/rows/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/rows/primary.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/identity/crate_name.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/identity/name.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/target_modes.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments/count.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments/tokens.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token/charset.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token/start.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token/underscore.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/capabilities/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/capabilities/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/capabilities/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/capabilities/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/capabilities/row/kind_prefix.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/capabilities/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/crate_name.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/crate_name/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/crate_name/token.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/crate_name/underscore.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/names.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/names/kind_suffix.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/names/owner_prefix.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/names/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/names/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes/row/coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes/row/editor_host.rs
  - zircon_runtime/src/plugin/runtime_plugin/module_validation/target_modes/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity/charset.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity/start.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity/underscore.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments/count.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments/tokens.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/token.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/token/charset.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/token/predicate.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/strategies.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/strategies/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/strategies/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers/provider_id.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets/coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets/module.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets/modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/kind.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/kind/feature_extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/kind/standard.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists/feature_extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists/optional.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/row/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/row/target_coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/interfaces.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/interfaces/dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/interfaces/exports.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata/owner.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata/version.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/row/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/capability.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/capability/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/capability/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/rows/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row/capability.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row/pair.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/note.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/owned_capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/note.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/references.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row/path.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/path.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/path/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/row/coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contributions/groups.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/components/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/components/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/event_catalogs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/event_catalogs/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/event_catalogs/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/options.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/options/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/options/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/ui_components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/ui_components/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/ui_components/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/components/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs/prefix.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/ui_components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/ui_components/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/fields.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/presence/completeness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/presence/fields.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/shape/prefix.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/shape/segment.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/description.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/public_metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_platforms.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_platforms/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_platforms/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_targets/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_targets/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/identity/crate_name.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/identity/name.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/target_modes.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/array.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/array/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/array/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/relative.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/separator.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component/digits.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component/leading_zeroes.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component/range.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments/count.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
implementation_files:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_kind.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_interface_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_module_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/extension_registry/validation.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/construction.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/fluent.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/runtime_module.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/system_anchors.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/capabilities/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/owner.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/primary_count.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/row/capability.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/row/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/rows/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/dependencies/rows/primary.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/identity/crate_name.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/identity/name.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/row/target_modes.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/modules/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments/count.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/namespace/segments/tokens.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token/charset.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token/start.rs
  - zircon_runtime/src/plugin/runtime_plugin/feature_validation/shape/token/underscore.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity/charset.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity/start.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/identity/underscore.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments/count.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/namespace/segments/tokens.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/token.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/token/charset.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/shape/token/predicate.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/strategies.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/strategies/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/default_packaging/strategies/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers/provider_id.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_providers/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets/coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets/module.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_feature_targets/modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/kind.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/kind/feature_extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/kind/standard.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists/feature_extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists/optional.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/lists/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/row/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/embedded_features/row/target_coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata/owner.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/metadata/version.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/identity/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/required_capabilities/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/row/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/capabilities/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/capability.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/capability/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/capability/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/rows/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row/capability.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row/pair.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/row/provider.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_dependencies/dependencies/pairs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity/namespace.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/identity/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/note.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/owned_capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/note.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/references.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/row/targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row/path.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/path.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/path/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_references/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/row/coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/row/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/coverage.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/capability_status_targets/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contributions.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contributions/groups.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/components/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/components/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/event_catalogs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/event_catalogs/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/event_catalogs/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/options.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/options/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/options/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/ui_components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/ui_components/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_duplicates/ui_components/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/components/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs/prefix.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/event_catalogs/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/ui_components.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/contribution_owners/ui_components/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/fields.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/presence.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/presence/completeness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/presence/fields.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/shape.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/shape/prefix.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/coordinates/shape/segment.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/description.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/public_metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_platforms.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_platforms/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_platforms/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_targets.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_targets/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/layout/supported_targets/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/identity.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/identity/crate_name.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/identity/name.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/row/target_modes.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/modules/rows/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/array.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/array/state.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/array/uniqueness.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/relative.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/separator.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/roots/path/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component/digits.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component/leading_zeroes.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/component/range.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/field.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/versions/segments/count.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
plan_sources:
  - user: 2026-05-03 review follow-up for plugin workspace compile failure
  - user: 2026-05-02 sound plugin mixer/spatial/convolution/timeline core implementation request
  - .codex/plans/Sound 插件核心完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/Zircon UI .zui 组件资产与 Unreal 风格入口重构计划.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/package_manifest_declarations.rs::plugin_package_manifest_declares_bridge_interfaces
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest.rs::native_runtime_plugin_registration_report_rejects_invalid_bridge_interface_declarations
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest.rs::native_runtime_plugin_registration_report_accepts_interface_only_dependency_rows
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/interfaces.rs
  - cargo test -p zircon_runtime --lib zui --locked (2026-05-14 .zui UI component descriptor suffix validation: planned for milestone testing stage)
  - cargo check -p zircon_runtime --lib --locked (2026-05-14 .zui plugin manifest boundary: planned for milestone testing stage)
  - cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --jobs 1
  - cargo check -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short (passed from zircon_plugins workspace with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-closeout)
  - cargo test -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short (passed from zircon_plugins workspace with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-closeout; 8 sound tests passed)
  - cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics --color never
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings after re-exporting PluginPackageKind, preserving feature diagnostics, and restoring external feature export helpers)
  - cargo test -p zircon_runtime --lib runtime_extension_registry_rejects_legacy_ui_component_documents --jobs 1 --target-dir target\codex-ui-v2-guard (2026-05-13: passed, 1 passed)
  - cargo test -p zircon_runtime --lib runtime_extension_registry_installs_ui_components_into_runtime_registry --jobs 1 --target-dir target\codex-ui-v2-guard (2026-05-13: passed, 1 passed)
  - cargo test -p zircon_runtime --lib plugin_package_manifest_declares_runtime_and_editor_contributions --jobs 1 --target-dir target\codex-ui-v2-guard (2026-05-13: passed, 1 passed)
  - cargo test -p zircon_runtime --lib builtin_runtime_catalog_optional_features_match_static_plugin_manifests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plugin-ecosystem-sync --message-format short --color never
  - cargo test -p zircon_runtime --lib importer_registry_rejects_non_fixture_legacy_ui_toml_importer_registration --jobs 1 --target-dir target\codex-ui-v2-guard (2026-05-13: passed, 1 passed)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never -- --test-threads=1 (2026-06-02 package capability/dependency validation subgroup: passed, 29 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-capability-rows --message-format short --color never (2026-06-02 package capability row subgroup: passed in 8m52s with 13 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-capability-row-dispatch --message-format short --color never (2026-06-02 package capability row dispatch subgroup: not run because active Workbench/asset Cargo and rustc processes were already compiling `zircon_editor`; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-capability-row-namespace --message-format short --color never (2026-06-03 package capability row namespace subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-capability-row-uniqueness-adapter --message-format short --color never (2026-06-03 package capability row uniqueness adapter subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-row-state --message-format short --color never (2026-06-03 package capability row state subgroup, verified together with option duplicate row state subgroup: passed in 11m42s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-capability-uniqueness --message-format short --color never (2026-06-02 package capability uniqueness subgroup: passed in 6m03s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-dependency-capability --message-format short --color never (2026-06-03 package dependency capability subgroup: not run because active workspace/asset and editor Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-dependency-rows --message-format short --color never (2026-06-02 package dependency row subgroup: did not validate this plugin slice; compilation stopped after 8m01s in active render-chain file `zircon_runtime/src/graphics/scene/resources/mod.rs:18` with E0365 private re-export of `PostProcessLutTextureResource`; 5 warnings emitted before the error)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-dependency-row-state --message-format short --color never (2026-06-03 package dependency row state subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-dependency-row-capability-adapter --message-format short --color never (2026-06-03 package dependency row capability adapter subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-dependency-row-pair-adapter --message-format short --color never (2026-06-03 package dependency row pair adapter subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-dependency-pairs --message-format short --color never (2026-06-02 package dependency pair subgroup: not run because active Workbench/asset Cargo and rustc processes were already compiling `zircon_editor`; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never -- --test-threads=1 (2026-06-02 capability-status Bevy reference path subgroup: passed, 29 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-reference-field --message-format short --color never (2026-06-03 capability-status Bevy reference field subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-reference-path-segments --message-format short --color never (2026-06-03 capability-status Bevy reference path segment subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-reference-row --message-format short --color never (2026-06-03 capability-status Bevy reference row dispatch subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-reference-row-adapters --message-format short --color never (2026-06-03 capability-status Bevy reference row adapter subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-reference-row-state --message-format short --color never (2026-06-03 capability-status Bevy reference row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never (2026-06-02 package layout array subgroup: passed with existing warnings)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-boundary --message-format short --color never -- --test-threads=1 (2026-06-02 package layout array subgroup: blocked before plugin tests by active render-chain test module `E0583` in `render_pass_executor_registry/tests.rs`)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-layout-description --message-format short --color never (2026-06-03 package layout description subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc/link processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-layout-array-uniqueness --message-format short --color never (2026-06-03 package layout target/platform uniqueness subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-layout-supported-target-state --message-format short --color never (2026-06-03 package layout supported-target state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-layout-supported-platform-state --message-format short --color never (2026-06-03 package layout supported-platform state subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-target-modules --message-format short --color never (2026-06-03 embedded-feature target module traversal subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-target-module --message-format short --color never (2026-06-03 embedded-feature target module dispatch subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-contribution-duplicates --message-format short --color never (2026-06-02 contribution duplicate subgroup: passed with existing warnings after initial target-dir and compile-timeout retries)
  - cargo test -p zircon_runtime --lib extension_registry_options --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-contribution-duplicates --message-format short --color never -- --test-threads=1 (2026-06-02 contribution duplicate subgroup: timed out during test-binary compilation/link; matching leftover processes stopped)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-option-duplicate-uniqueness --message-format short --color never (2026-06-03 option duplicate uniqueness subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-row-state --message-format short --color never (2026-06-03 option duplicate row state subgroup, verified together with package capability row state subgroup: passed in 11m42s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-event-catalog-duplicate-uniqueness --message-format short --color never (2026-06-03 event catalog duplicate uniqueness subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-event-catalog-duplicate-row-state --message-format short --color never (2026-06-03 event catalog duplicate row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-component-duplicate-uniqueness --message-format short --color never (2026-06-03 component duplicate uniqueness subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-component-duplicate-row-state --message-format short --color never (2026-06-03 component duplicate row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-ui-component-duplicate-uniqueness --message-format short --color never (2026-06-03 UI component duplicate uniqueness subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-ui-component-duplicate-row-state --message-format short --color never (2026-06-03 UI component duplicate row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-contribution-owners --message-format short --color never (2026-06-02 contribution owner subgroup: passed with 20 existing warnings after an initial 304s timeout and warmed rerun)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-contribution-owners --message-format short --color never -- --test-threads=1 (2026-06-02 contribution owner subgroup: timed out after 484s during dependency/test-binary compilation; matching leftover processes stopped)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-component-owner-ownership --message-format short --color never (2026-06-03 component owner ownership subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-event-catalog-owner-ownership --message-format short --color never (2026-06-03 event catalog owner ownership subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-event-catalog-owner-prefix --message-format short --color never (2026-06-03 event catalog owner prefix subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-ui-component-owner-ownership --message-format short --color never (2026-06-03 UI component owner ownership subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-contribution-groups --message-format short --color never (2026-06-02 package contribution group subgroup: passed in 7m39s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status --message-format short --color never (2026-06-02 capability-status row subgroup: blocked before validating plugin files by active render-chain syntax error in `graphics/scene/scene_renderer/sprite/prepared_batches.rs:51`)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-identity --message-format short --color never (2026-06-02 capability-status identity subgroup: passed in 7m22s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-identity-rules --message-format short --color never (2026-06-03 capability-status identity rule subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc/link processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-identity-namespace --message-format short --color never (2026-06-03 capability-status identity namespace subgroup: not run because active shared-checkout build processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-row-dispatch --message-format short --color never (2026-06-02 capability-status row dispatch subgroup: passed in 8m03s with 14 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-note --message-format short --color never (2026-06-03 capability-status note subgroup: not run because active workspace/asset Cargo, rustc, and linker processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-row-adapters --message-format short --color never (2026-06-03 capability-status row adapter subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-row-state --message-format short --color never (2026-06-03 capability-status row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-root-path-segments --message-format short --color never (2026-06-03 root path segment subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-root-path-field --message-format short --color never (2026-06-03 root path field subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-root-path-relative --message-format short --color never (2026-06-03 root path relative subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-root-path-separator --message-format short --color never (2026-06-03 root path separator subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-root-array-uniqueness --message-format short --color never (2026-06-03 root array uniqueness subgroup: not run because active shared-checkout build processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-root-array-state --message-format short --color never (2026-06-03 root array state subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-kind-subgroups --message-format short --color never (2026-06-03 embedded feature kind subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-modules --message-format short --color never (2026-06-02 package module row subgroup: passed with 12 existing warnings)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-modules --message-format short --color never -- --test-threads=1 (2026-06-02 package module row subgroup: timed out after 604s without a test result and left no matching processes)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-module-row --message-format short --color never (2026-06-02 package module single-row subgroup: passed in 7m26s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-module-row-state --message-format short --color never (2026-06-03 package module row state subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-module-identity --message-format short --color never (2026-06-02 package module identity subgroup: passed in 8m27s with 13 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-module-identity-rules --message-format short --color never (2026-06-03 package module identity rule subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importers --message-format short --color never (2026-06-02 asset-importer row subgroup: passed in 8m16s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-identity --message-format short --color never (2026-06-02 asset-importer identity subgroup: passed in 7m37s with 13 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-identity-namespace --message-format short --color never (2026-06-03 asset-importer identity namespace subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-metadata-subgroups --message-format short --color never (2026-06-03 asset-importer metadata owner/version subgroup: passed in 9m49s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-row-dispatch --message-format short --color never (2026-06-02 asset-importer row dispatch subgroup: not run because active runtime/editor Cargo and rustc processes from other lanes were running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked-file diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-row-state --message-format short --color never (2026-06-03 asset-importer row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-required-capability-state --message-format short --color never (2026-06-03 asset-importer required-capability state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-required-capability-uniqueness --message-format short --color never (2026-06-03 asset-importer required-capability uniqueness subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importer-required-capability-namespace --message-format short --color never (2026-06-03 asset-importer required-capability namespace subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-manifest --message-format short --color never (2026-06-03 embedded-feature manifest subgroup: not run because active workspace/asset and render-main-chain Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-identity --message-format short --color never (2026-06-03 feature manifest identity subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-asset-importers --message-format short --color never -- --test-threads=1 (2026-06-02 asset-importer row subgroup: not rerun because the adjacent package-module run timed out after 604s in the shared checkout)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-versions --message-format short --color never (2026-06-02 package version component boundary: passed in 8m14s with 12 existing warnings)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-versions --message-format short --color never -- --test-threads=1 (2026-06-02 package version component boundary: not rerun because this is a behavior-preserving split and adjacent package-manifest focused tests have timed out during test-binary compilation)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-version-segments --message-format short --color never (2026-06-03 package version segment subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-version-segment-count --message-format short --color never (2026-06-03 package version segment-count subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-version-field --message-format short --color never (2026-06-03 package version field subgroup: not run because active shared-checkout build processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-version-component-rules --message-format short --color never (2026-06-03 package version component rule subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-shape-namespace-segments --message-format short --color never (2026-06-03 package namespace segment rule subgroup: not run because active workspace/asset, render-main-chain, and Hub Cargo/rustc/link processes were already running; implementation-stage evidence is focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-default-packaging --message-format short --color never (2026-06-02 default-packaging strategy subgroup: blocked before validating plugin files by active render-chain `PostProcessParams` missing-field initializer error in `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs:22`)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-default-packaging-strategy-state --message-format short --color never (2026-06-03 default-packaging strategy state subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-default-packaging-presence --message-format short --color never (2026-06-02 default-packaging presence subgroup: passed in 8m39s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-coordinate-presence --message-format short --color never (2026-06-02 package coordinate presence subgroup: passed in 7m51s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-coordinate-fields --message-format short --color never (2026-06-02 package coordinate fields subgroup: passed in 8m00s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-coordinate-prefix --message-format short --color never (2026-06-03 package coordinate prefix subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-coordinate-segment --message-format short --color never (2026-06-03 package coordinate segment subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-coordinate-presence --message-format short --color never (2026-06-03 package coordinate presence subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-identity-start --message-format short --color never (2026-06-03 package identity start subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-identity-charset --message-format short --color never (2026-06-03 package identity charset subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-identity-underscore --message-format short --color never (2026-06-03 package identity underscore subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-token-charset --message-format short --color never (2026-06-03 package token charset subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-package-token-predicate --message-format short --color never (2026-06-03 package token predicate subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-shape --message-format short --color never (2026-06-02 feature shape subgroup: passed in 7m56s with 12 existing warnings)
  - cargo test -p zircon_runtime --lib runtime_plugin_feature_descriptor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-shape --message-format short --color never -- --test-threads=1 (2026-06-02 feature shape subgroup: timed out after 604s without a test result and left no matching target-dir cargo/rustc/link processes)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-namespace-segments --message-format short --color never (2026-06-03 feature namespace segment subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-token-start --message-format short --color never (2026-06-03 feature token start subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-token-charset --message-format short --color never (2026-06-03 feature token charset subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-token-underscore --message-format short --color never (2026-06-03 feature token underscore subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-targets --message-format short --color never (2026-06-02 capability-status target subgroup: first run reached compile finish but hit the 603s tool timeout; warmed rerun passed in 30.94s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-target-rows --message-format short --color never (2026-06-02 capability-status target row subgroup: passed in 7m46s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-target-row --message-format short --color never (2026-06-03 capability-status target row dispatch subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-target-row-adapters --message-format short --color never (2026-06-03 capability-status target row adapter subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-target-row-state --message-format short --color never (2026-06-03 capability-status target row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-targets --message-format short --color never -- --test-threads=1 (2026-06-02 capability-status target subgroup: timed out after 604s without a test result and left no matching target-dir cargo/rustc/link processes)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-provider --message-format short --color never (2026-06-02 embedded-feature provider subgroup: passed in 8m48s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-targets --message-format short --color never (2026-06-02 embedded-feature target coverage subgroup: passed in 9m38s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-row --message-format short --color never (2026-06-02 embedded-feature row subgroup: passed in 7m55s with 13 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-list-state --message-format short --color never (2026-06-03 embedded-feature list state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-embedded-feature-list-kind --message-format short --color never (2026-06-03 embedded-feature list-kind subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-capabilities --message-format short --color never (2026-06-02 feature capability row subgroup: passed in 7m44s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-capability-presence --message-format short --color never (2026-06-02 feature capability presence subgroup: passed in 7m50s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-capability-row-dispatch --message-format short --color never (2026-06-03 feature capability row dispatch subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-capability-row-state --message-format short --color never (2026-06-03 feature capability row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-dependency-rows --message-format short --color never (2026-06-02 feature dependency row subgroup: blocked before validating plugin files by active render-chain errors in `render_pass_execution_context/gpu.rs:397` and `builtin_scene_executors.rs:16` / `:60`)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-dependency-row-adapters --message-format short --color never (2026-06-03 feature dependency row adapter subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-dependency-presence --message-format short --color never (2026-06-03 feature dependency presence subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-dependency-primary-count --message-format short --color never (2026-06-03 feature dependency primary-count subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-dependency-row-state --message-format short --color never (2026-06-03 feature dependency row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-provider --message-format short --color never (2026-06-02 feature provider validation subgroup: passed in 7m50s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-modules --message-format short --color never (2026-06-02 feature module row subgroup: passed in 8m11s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-module-row-dispatch --message-format short --color never (2026-06-03 feature module row dispatch subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-module-row-state --message-format short --color never (2026-06-03 feature module row state subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-module-row-adapters --message-format short --color never (2026-06-03 feature module row adapter subgroup: not run because active shared-checkout Cargo/rustc processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene are the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-feature-module-row-identity --message-format short --color never (2026-06-03 feature module row identity subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, hard-cutover stale-path scan, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-module-names --message-format short --color never (2026-06-03 shared module name validation subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-module-capabilities --message-format short --color never (2026-06-03 shared module capability validation subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-module-target-modes --message-format short --color never (2026-06-03 shared module target-mode validation subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-module-crate-name --message-format short --color never (2026-06-03 shared module crate-name validation subgroup: not run because active shared-checkout Cargo/rustc/link processes were already running; focused rustfmt, migration-word scan, direct whitespace/conflict scans, and tracked docs diff hygiene passed as the implementation-stage gate)
  - rustfmt --edition 2021 --check zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\modules.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\modules\capabilities.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\modules\crates.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\modules\identity.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\modules\names.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\modules\targets.rs (2026-06-03 static manifest module contract test-tree boundary: passed)
  - cargo test -p zircon_runtime --lib plugin_tomls_declare_module --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-static-modules-boundary --message-format short --color never -- --test-threads=1 (2026-06-03 static manifest module contract test-tree boundary: timed out after 604s during compile without a test result; no matching target-dir processes remained, and the active asset workspace Cargo/rustc lane prevented a warmed rerun)
  - rustfmt --edition 2021 --check zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\options.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\options\enums.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\options\keys.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\options\rows.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\options\shape.rs zircon_runtime\src\tests\plugin_extensions\static_manifest_contracts\options\traversal.rs (2026-06-03 static manifest option contract test-tree boundary: passed)
  - cargo check/test for the 2026-06-03 static manifest option contract test-tree boundary: deferred because active shared-checkout Cargo/rustc lanes were already running for asset workspace validation, editor style-selector tests, and render-main-chain tests; implementation-stage evidence is focused rustfmt, conflict-marker scan, and tracked diff hygiene.
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-references --message-format short --color never (2026-06-02 capability-status reference uniqueness subgroup: passed in 9m07s with 12 existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-plugin-capability-status-reference-rows --message-format short --color never (2026-06-02 capability-status reference row subgroup: passed in 7m50s with 12 existing warnings)
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules/capabilities.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules/crates.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules/identity.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules/names.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/modules/targets.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/options.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/options/enums.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/options/keys.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/options/rows.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/options/shape.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/options/traversal.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/manifest_schema.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/manifest_schema/assertions.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/manifest_schema/field_sets.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/manifest_schema/nested.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/manifest_schema/top_level.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_layout.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_layout/arrays.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_layout/default_packaging.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_layout/roots.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_layout/supported_platforms.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_kind.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_kind/feature_rows.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_kind/helpers.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_kind/values.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_identity.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_identity/directories.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_identity/helpers.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_identity/namespaces.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_identity/uniqueness.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_coordinates.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_coordinates/coordinates.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_coordinates/helpers.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_coordinates/resolved_ids.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_metadata.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_metadata/arrays.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_metadata/classification.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_metadata/display.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_metadata/public_fields.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_metadata/targets.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_versions.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_versions/semantic.rs
  - zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/package_versions/semver.rs
doc_type: module-detail
---

# Plugin Package Manifest Extensions

## Purpose

`PluginPackageManifest` describes what a plugin package contributes before any concrete Rust runtime/editor code is activated. It carries SDK/API version, package category, supported targets/platforms, capabilities, asset/content roots, module declarations, component/UI component metadata, asset importer descriptors, optional feature bundles, and packaging metadata. The sound core and independent plugin slices add neutral manifest contribution kinds that are needed by independent plugins:

- dependencies,
- options,
- event catalogs,
- asset importer descriptors.

These fields are generic because sound is not the only plugin that needs optional feature gates, project-visible settings, or future event namespaces.

## Behavior Model

`PluginDependencyManifest` records another plugin, capability, or bridge interface that this package expects. `required = true` means the package cannot fully operate without that dependency. `required = false` means the package exposes a gated advanced path when the capability or interface exists. Capability dependencies keep using the optional `capability` field. Bridge dependencies use the serde-defaulted `interfaces` list, so dependency rows can be capability-only, interface-only, or both.

`PluginInterfaceManifest` records bridge interfaces exported by this package through the top-level `provides_interfaces` TOML array. Each row has a stable dot-namespaced `id`, for example `physics.query.v1`. These manifest rows are declaration metadata; the runtime still requires concrete code to call `RuntimeExtensionRegistry::export_interface::<T>(...)` during registration before a `StrongBridge` or `WeakBridge` can resolve the provider.

`PluginOptionManifest` records editor/project-visible configuration metadata. Values are stored as strings in the manifest so the manifest remains a simple TOML contract and does not depend on a runtime value enum. Consumers should parse values according to `value_type`.

The runtime extension registry validates option rows before installation. Option keys and required capabilities are dot-namespaced lowercase tokens. `value_type` is limited to `bool`, `integer`, `number`, `string`, and `enum`; defaults must be non-empty and parse according to the declared type. Enum options must declare a non-empty `enum_values` list, every enum token must be lowercase ASCII with digits, underscores, or hyphens, values must be unique, and the default must be one of those values. Non-enum options must not carry `enum_values`.

Static manifest option contract tests are folder-backed to keep row traversal, key shape, and enum/default-value rules separate. `static_manifest_contracts/options.rs` is child-module wiring only. `options/rows.rs` owns the full option row contract and global key uniqueness, `options/keys.rs` owns the focused key namespace contract, `options/enums.rs` owns focused enum default/value tests, `options/shape.rs` owns shared trim, key namespace, value type, default parsing, and enum-value helpers, and `options/traversal.rs` owns TOML option array traversal. This mirrors the production validation boundaries without changing test names, diagnostics, or validation order.

Static manifest schema tests are also folder-backed. `static_manifest_contracts/manifest_schema.rs` is child-module wiring only, `manifest_schema/field_sets.rs` owns the top-level and nested `PluginPackageManifest` field allow-lists, `manifest_schema/assertions.rs` owns shared unknown-field assertions plus component and feature-bundle row adapters, `manifest_schema/top_level.rs` owns the top-level unknown-field contract, and `manifest_schema/nested.rs` owns nested row-field coverage for dependencies, modules, options, components, UI components, asset importers, capability statuses, event catalogs, optional features, and feature-extension rows. The split keeps typo rejection visible before plugin-window, catalog projection, export selection, or native package loading consume static plugin TOML.

Bridge interface manifest validation is folder-backed under `package_validation/interfaces.rs`. `interfaces/exports.rs` owns `provides_interfaces` namespace and uniqueness diagnostics. `interfaces/dependencies.rs` owns dependency `interfaces` namespace and per-dependency uniqueness diagnostics. Package dependency capability validation now treats `capability` and `interfaces` as alternative dependency payloads: a row must declare at least one, but an interface-only bridge dependency no longer needs a capability placeholder.

Linked Rust plugin registration also validates that package interface declarations match the runtime registry. `registration_report/validation/interfaces.rs` checks declared-but-unexported ids from `provides_interfaces` and exported-but-undeclared ids from `RuntimeExtensionRegistry::export_interface(...)`. This validation is registration-report scoped; it does not yet resolve graph-wide strong dependency closure or produce cross-plugin dependency chains.

`PluginEventCatalogManifest` records a namespaced event catalog with a positive version and at least one event. Catalog namespaces must use lowercase dot-separated package namespaces. Event ids must stay under the catalog namespace, display names must be non-empty, duplicate ids are rejected, and payload schemas, when present, must stay under the package namespace and end in a positive version segment such as `v1`.

`RuntimeExtensionRegistry` mirrors options, event catalogs, manifest-declared components, UI components, and asset importer descriptors during linked plugin registration so runtime/editor hosts can discover them alongside modules, managers, render features, pass executors, and runtime providers. If a plugin has already registered a real importer backend with the same importer id, the manifest descriptor is treated as the public descriptor for that backend and the registration report does not add the diagnostic-only placeholder.

Runtime UI component manifest entries are `.zui`-only on the production path. The registry still accepts the lightweight `{ component_id, plugin_id, ui_document }` shape, but `register_ui_component(...)` now rejects documents that do not end in `.zui`. This keeps plugin component metadata from reintroducing recursive `.ui.toml` prototypes or broad `.v2.ui.toml` documents after the UI component asset cutover.

Asset importer manifest entries are `.zui`-only for production UI component documents as well. `AssetImporterRegistry` rejects any non-fixture importer descriptor whose full suffix is `.ui.toml` or `.v2.ui.toml`, so stale package manifests cannot reinstall the recursive `UiAssetDocument` importer or the old mixed-kind v2 importer even when they declare an otherwise valid asset importer contribution. The only surviving old matchers are unit-test migration fixtures used to verify old-schema migration metadata.

Asset importer package validation is folder-backed. `package_validation/asset_importers.rs`
keeps the internal entry point, `asset_importers/rows.rs` owns list traversal,
`asset_importers/rows/state.rs` owns seen-id state creation and lifetime typing,
`asset_importers/row.rs` owns per-importer identity and required-capability dispatch,
`asset_importers/identity.rs` owns the identity entry point, `identity/metadata.rs` owns
metadata rule dispatch, `identity/metadata/namespace.rs` owns importer id namespace diagnostics,
`identity/metadata/owner.rs` owns owner plugin-id token and package-match diagnostics,
`identity/metadata/version.rs` owns positive importer-version diagnostics,
`identity/uniqueness.rs` owns package-local duplicate-id diagnostics,
`asset_importers/required_capabilities.rs` owns
required-capability traversal and rule dispatch,
`asset_importers/required_capabilities/namespace.rs` owns required-capability namespace diagnostics,
`asset_importers/required_capabilities/state.rs` owns seen-required-capability state creation and
lifetime typing, and `asset_importers/required_capabilities/uniqueness.rs` owns duplicate
required-capability diagnostics plus seen insertion before catalog projection or linked/native
registration reports consume importer metadata.

Package manifests and feature manifests expose the same `with_default_packaging(...)` builder shape. That lets standalone plugin packages, such as editor-only export plugins, override the package-level default export strategy without reaching into the public struct fields or relying on feature-bundle builders by mistake.

First-party owner-embedded optional-feature rows are declared in static `zircon_plugins/<owner>/plugin.toml` manifests and mirrored by `RuntimePluginDescriptor::builtin_catalog()` for runtime/project projection. The mirror must stay byte-for-byte equivalent after TOML decoding for feature id, owner id, dependencies, module rows, capabilities, default packaging, and default enablement. This prevents the export planner and editor Plugin Manager from seeing a different optional-feature contract than the package manifest that ships with the plugin.

Runtime plugin default-packaging validation is folder-backed. `package_validation/default_packaging.rs` keeps the shared descriptor/package/feature entry point, `default_packaging/presence.rs` owns empty-list diagnostics, `default_packaging/strategies.rs` owns strategy list traversal, `default_packaging/strategies/state.rs` owns seen-strategy state creation, and `default_packaging/strategies/uniqueness.rs` owns duplicate strategy diagnostics plus seen-strategy insertion before linked descriptors, native package manifests, and feature manifests choose export packaging fallbacks.

Feature-manifest validation is folder-backed because package manifests can embed optional-feature and feature-extension rows. `feature_validation.rs` keeps the internal feature-manifest entry surface and provider package-id entry surface, while `feature_validation/identity.rs` owns feature id field/namespace diagnostics, display-name field diagnostics, owner plugin-id field/token diagnostics, and owner-prefix diagnostics before capability, dependency, module, and default-packaging validation run. `feature_validation/shape.rs` keeps the internal helper surface, while `shape/field.rs`, `shape/namespace.rs`, and `shape/token.rs` own trimmed field, feature namespace, and owner/provider token diagnostics before package-embedded feature rows feed catalog projection. Namespace validation is split so `shape/namespace.rs` keeps namespace entry dispatch, `shape/namespace/segments.rs` owns segment-rule dispatch, `shape/namespace/segments/count.rs` owns minimum two-segment diagnostics, and `shape/namespace/segments/tokens.rs` owns lowercase segment diagnostics. Token validation is itself folder-backed: `shape/token.rs` owns token rule orchestration, `shape/token/start.rs` owns lowercase-start diagnostics, `shape/token/charset.rs` owns lowercase ASCII letters/digits/underscore character-set diagnostics, and `shape/token/underscore.rs` owns trailing/repeated underscore diagnostics.

Package-manifest shape validation uses the same folder-backed pattern. `package_validation/shape.rs` keeps the internal helper surface, `shape/field.rs` owns trimmed/non-empty field diagnostics, `shape/token.rs` owns package-token rule orchestration, `shape/token/charset.rs` owns package-token trim and lowercase ASCII character-set diagnostics, `shape/token/predicate.rs` owns the shared lowercase token predicate, `shape/identity.rs` owns package-id rule orchestration, `shape/identity/charset.rs` owns package-id lowercase ASCII character-set diagnostics, `shape/identity/start.rs` owns package-id leading-letter diagnostics, `shape/identity/underscore.rs` owns package-id trailing/repeated underscore diagnostics, `shape/namespace.rs` owns namespace field gating, `shape/namespace/segments.rs` owns segment-rule dispatch, `shape/namespace/segments/count.rs` owns minimum segment diagnostics, and `shape/namespace/segments/tokens.rs` owns lowercase segment diagnostics before descriptor, native package, package feature, and module validation consume the shared package-shape contract.

Feature provider validation is also folder-backed. `feature_validation/provider.rs` owns `provider_package_id` field and token diagnostics consumed by native feature manifests and provider overrides, while `feature_validation.rs` keeps the internal provider entry surface.

Feature capability validation follows the same owner/child shape. `feature_validation/capabilities.rs` keeps the feature-level entry point, `feature_validation/capabilities/presence.rs` owns empty-list diagnostics, `feature_validation/capabilities/rows.rs` owns list traversal, `feature_validation/capabilities/rows/state.rs` owns seen-capability state creation, `feature_validation/capabilities/row.rs` owns per-capability validation order, `feature_validation/capabilities/row/field.rs` owns field-validation adapter dispatch, `feature_validation/capabilities/row/namespace.rs` owns namespace-validation adapter dispatch, `feature_validation/capabilities/row/uniqueness.rs` owns feature-local duplicate adapter dispatch, and `feature_validation/capabilities/uniqueness.rs` owns duplicate diagnostics before feature rows feed dependency and package-embedded validation.

Feature dependency validation is folder-backed as well. `feature_validation/dependencies.rs`
keeps the feature-level entry point, `dependencies/presence.rs` owns empty-list diagnostics,
and `dependencies/rows.rs` owns dependency list traversal plus per-row validation order. Row
state is split under that traversal owner: `dependencies/rows/pairs.rs` owns duplicate-pair
seen-state adaptation into `dependencies/pairs.rs`, and `dependencies/rows/primary.rs` owns
primary dependency count accumulation plus final count dispatch into `dependencies/primary_count.rs`.
Row-level shape validation is split under `dependencies/row.rs`, which owns dispatch order,
while `dependencies/row/provider.rs` owns provider id field/token diagnostics and
`dependencies/row/capability.rs` owns dependency capability field/namespace diagnostics. The
duplicate-pair, primary-owner, and exactly-one-primary rule implementations stay in
`dependencies/pairs.rs`, `dependencies/owner.rs`, and `dependencies/primary_count.rs`.

Shared module validation is folder-backed under `module_validation.rs` so package and feature module rows consume one internal contract. `module_validation/names.rs` keeps the shared module-name entry point, `module_validation/names/shape.rs` owns caller-provided field and namespace validation dispatch, `module_validation/names/owner_prefix.rs` owns package/feature owner-prefix diagnostics, `module_validation/names/kind_suffix.rs` owns runtime/editor suffix diagnostics, and `module_validation/names/uniqueness.rs` owns package/feature-local duplicate-name diagnostics plus seen insertion. `module_validation/capabilities.rs` keeps the shared capability entry point, `module_validation/capabilities/presence.rs` owns empty capability-list diagnostics, `module_validation/capabilities/rows.rs` owns capability traversal, `module_validation/capabilities/rows/state.rs` owns seen-capability state creation, `module_validation/capabilities/row.rs` owns per-capability validation order, `module_validation/capabilities/row/kind_prefix.rs` owns runtime/editor capability-prefix diagnostics, and `module_validation/capabilities/row/uniqueness.rs` owns duplicate capability diagnostics plus seen insertion. `module_validation/target_modes.rs` keeps the shared target-mode entry point, `module_validation/target_modes/presence.rs` owns empty target-mode diagnostics, `module_validation/target_modes/rows.rs` owns target-mode traversal, `module_validation/target_modes/rows/state.rs` owns seen-target state creation, `module_validation/target_modes/row.rs` owns per-target validation order, `module_validation/target_modes/row/uniqueness.rs` owns duplicate target diagnostics plus seen insertion, `module_validation/target_modes/row/editor_host.rs` owns editor-only `EditorHost` diagnostics, and `module_validation/target_modes/row/coverage.rs` owns optional package supported-target coverage diagnostics. `module_validation/crate_name.rs` keeps the shared module crate-name entry point, `module_validation/crate_name/shape.rs` owns caller-provided field validation dispatch, `module_validation/crate_name/token.rs` owns `zircon_plugin_` prefix and lowercase token diagnostics, and `module_validation/crate_name/underscore.rs` owns trailing/repeated underscore diagnostics.

Feature module validation is also folder-backed. `feature_validation/modules.rs` keeps the feature-level module entry point, `feature_validation/modules/rows.rs` owns module list traversal, `feature_validation/modules/rows/state.rs` owns seen-name state creation, and `feature_validation/modules/row.rs` owns per-module validation order. The row adapters are split by shared-rule family: `modules/row/identity.rs` owns identity rule dispatch, `modules/row/identity/name.rs` owns feature-id scoped module-name validation, `modules/row/identity/crate_name.rs` owns module crate-name validation, `modules/row/capabilities.rs` owns feature field/namespace adaptation for shared module capability checks, and `modules/row/target_modes.rs` owns target-mode dispatch for feature modules.

Static manifest module contract tests mirror that module-validation ownership instead of hiding helper logic in the test root. `static_manifest_contracts/modules.rs` is now child-module wiring only. `modules/identity.rs` owns identity and non-empty capability/target list checks, `modules/names.rs` owns package/optional-feature namespace and suffix checks, `modules/crates.rs` owns `zircon_plugins/Cargo.toml` workspace member resolution plus crate-name/member-path shape helpers, `modules/capabilities.rs` owns kind-matching capability namespace checks, and `modules/targets.rs` owns package target coverage and editor-host-only checks. The split preserves the existing static module contract names, assertion text, and validation order while making the test tree show the same responsibility boundaries as the production module-validation tree.

Embedded feature package validation keeps optional-feature and feature-extension row flow split by responsibility. `package_validation/embedded_features.rs` owns the package-level entry point and package-kind dispatch order, `embedded_features/kind.rs` owns `package_kind` dispatch, `embedded_features/kind/standard.rs` owns Standard package diagnostics, `embedded_features/kind/feature_extension.rs` owns FeatureExtension package diagnostics, `embedded_features/lists.rs` owns list-kind dispatch, `embedded_features/lists/state.rs` owns the shared provider seen-state factory and lifetime type, `embedded_features/lists/optional.rs` owns optional-feature traversal, `embedded_features/lists/feature_extension.rs` owns feature-extension traversal, `embedded_features/row.rs` owns per-feature dispatch order, `embedded_features/manifest.rs` owns runtime feature-manifest validation dispatch, `embedded_features/row/provider.rs` owns provider validation dispatch, and `embedded_features/row/target_coverage.rs` owns target-coverage dispatch. Provider validation stays under `embedded_feature_providers.rs`, which now delegates provider package-id resolution to `embedded_feature_providers/provider_id.rs` and duplicate provider rows to `embedded_feature_providers/uniqueness.rs`; `embedded_feature_targets.rs` keeps the internal target coverage entry point, `embedded_feature_targets/modules.rs` owns feature module traversal, `embedded_feature_targets/module.rs` owns per-module target-mode dispatch, and `embedded_feature_targets/coverage.rs` owns per-target package supported-target coverage diagnostics.

`PluginPackageKind` is part of the top-level `crate::plugin` public surface. Native plugin load
projection and runtime catalog feature-definition logic both consume it through that surface, so
the package-kind enum must be re-exported next to `PluginPackageManifest` rather than only from the
private package-manifest subtree.

Static manifest package-kind tests keep that public enum boundary explicit. `static_manifest_contracts/package_kind.rs`
is child-module wiring only, `package_kind/helpers.rs` owns the defaulted package-kind value lookup
and feature-row table counting helpers, `package_kind/values.rs` owns the known-value contract, and
`package_kind/feature_rows.rs` owns the Standard versus FeatureExtension row-shape coherence checks.
This keeps static TOML package-kind diagnostics separate from feature-extension row traversal.

Static manifest package-identity tests are folder-backed as well. `static_manifest_contracts/package_identity.rs`
is child-module wiring only, `package_identity/helpers.rs` owns the package-id token contract,
`package_identity/directories.rs` owns package-id-to-directory matching, `package_identity/namespaces.rs`
owns optional-feature dot-namespace and owner-prefix checks, and `package_identity/uniqueness.rs`
owns static package and optional-feature id uniqueness. This keeps identity token shape, namespace
shape, directory ownership, and global uniqueness assertions separate without changing manifest
behavior.

Runtime plugin package version validation is folder-backed. `package_validation/versions.rs` keeps
the shared semver entry point and dispatch order for package `version` and `sdk_api_version`,
`versions/field.rs` owns trimmed-field validation, `versions/segments.rs` owns semver component
traversal, `versions/segments/count.rs` owns the `MAJOR.MINOR.PATCH` segment-count diagnostic,
`versions/component.rs` owns per-component rule dispatch,
`versions/component/digits.rs` owns ASCII digit diagnostics, `versions/component/leading_zeroes.rs`
owns leading-zero diagnostics, and `versions/component/range.rs` owns `u32` range diagnostics. This
keeps strict package/API version metadata aligned for linked descriptors and native package manifests
without growing the registration-report validation flow.

Static manifest package-version tests mirror that shape. `static_manifest_contracts/package_versions.rs`
is child-module wiring only, `package_versions/semantic.rs` owns package `version` and
`sdk_api_version` traversal, and `package_versions/semver.rs` owns the shared `MAJOR.MINOR.PATCH`
field, segment, digit, leading-zero, and `u32` range assertions. This keeps static TOML version
metadata checks aligned with runtime package-version validation without making the root test file a
helper owner.

Runtime plugin package validation keeps package-level capability and dependency diagnostics under
`runtime_plugin/package_validation/capability_dependencies.rs`. The owner exposes the same internal
registration-report entry points while `capability_dependencies/capabilities.rs` keeps the package
capability entry point, `capabilities/presence.rs` validates the non-empty package capability list,
`capabilities/rows.rs` owns package capability traversal, `capabilities/rows/state.rs` owns
seen-capability state creation and lifetime typing,
`capabilities/row.rs` owns per-capability validation order,
`capabilities/row/namespace.rs` validates lowercase package capability namespaces,
`capabilities/row/uniqueness.rs` delegates package-local duplicate checks,
`capabilities/uniqueness.rs` owns duplicate package capability diagnostics and seen-capability insertion,
`capability_dependencies/dependencies.rs` keeps the package dependency entry point,
`dependencies/rows.rs` owns dependency row traversal,
`dependencies/rows/pairs.rs` owns the seen `(id, capability)` pair state lifetime,
`dependencies/row.rs` owns dependency row dispatch order,
`dependencies/row/provider.rs` validates provider ids,
`dependencies/row/capability.rs` delegates required-capability validation,
`dependencies/row/pair.rs` delegates duplicate dependency-pair validation,
`dependencies/capability.rs` owns required-capability dispatch,
`dependencies/capability/presence.rs` owns missing required-capability diagnostics,
`dependencies/capability/namespace.rs` owns required-capability namespace diagnostics,
and `dependencies/pairs.rs` owns duplicate dependency-pair diagnostics plus seen insertion. This keeps programmatic
and native package manifests on the same strict metadata contract as static plugin manifests before
catalog dependency reports or export planning consume package capability data.

Capability-status package validation uses a folder-backed owner. `package_validation/capability_status.rs`
keeps the internal entry point, `capability_status/owned_capabilities.rs` collects capabilities declared
by the package and its optional features, `capability_status/identity.rs` owns identity-rule dispatch,
`capability_status/identity/namespace.rs` owns per-status capability namespace diagnostics,
`capability_status/identity/ownership.rs` owns package-owned capability diagnostics,
`capability_status/identity/uniqueness.rs` owns package-local duplicate status diagnostics,
`capability_status/rows.rs` owns row traversal,
`capability_status/rows/state.rs` owns seen-capability state creation and lifetime typing, and
`capability_status/row.rs` owns per-status dispatch order. `capability_status/row/identity.rs`
adapts the status row into identity validation, `capability_status/row/targets.rs` adapts it into
target coverage validation, `capability_status/row/references.rs` adapts it into Bevy source-reference
validation, and `capability_status/row/note.rs` adapts it into note validation.
`capability_status/note.rs` owns optional note field diagnostics. This keeps row-level capability-status
identity validation separate from target-mode, source-traceability, and text-field helper modules.

Capability-status target validation is split under `package_validation/capability_status_targets.rs`.
The owner keeps the internal entry point, `capability_status_targets/rows.rs` owns status target-mode
traversal, `capability_status_targets/rows/state.rs` owns seen-state creation and lifetime typing,
and `capability_status_targets/row.rs` owns per-target validation order.
`capability_status_targets/row/uniqueness.rs` adapts target rows into duplicate-target validation,
`capability_status_targets/row/coverage.rs` adapts target rows into package supported-target coverage
validation, `capability_status_targets/uniqueness.rs` owns duplicate target diagnostics, and
`capability_status_targets/coverage.rs` owns package supported-target coverage checks.

Capability-status Bevy reference validation keeps the same folder-backed package-validation shape.
`runtime_plugin/package_validation/capability_status_references.rs` owns the internal entry point,
`capability_status_references/rows.rs` owns Bevy reference list traversal,
`capability_status_references/rows/state.rs` owns seen-reference state creation and lifetime typing,
`capability_status_references/row.rs` owns per-reference validation order,
`capability_status_references/row/field.rs` adapts reference rows into package field validation,
`capability_status_references/row/path.rs` adapts reference rows into repository path validation,
`capability_status_references/row/uniqueness.rs` adapts reference rows into per-capability uniqueness
validation,
`capability_status_references/field.rs` owns package field diagnostics,
`capability_status_references/path.rs` owns repository-relative `dev/bevy` path shape dispatch,
`capability_status_references/path/segments.rs` owns empty/current/parent segment diagnostics, and
`capability_status_references/uniqueness.rs` owns
per-capability duplicate reference diagnostics.
This keeps source-traceability metadata strict without expanding the package-validation owner file.

Package layout validation follows the same owner/child split. `package_validation/layout.rs`
orchestrates public metadata, coordinates, target/platform arrays, and roots, while
`layout/public_metadata.rs` owns category field validation plus description-rule dispatch,
`layout/description.rs` owns optional description trim diagnostics,
`layout/supported_targets.rs` owns supported target traversal,
`layout/supported_targets/state.rs` owns seen-target state creation,
`layout/supported_targets/uniqueness.rs` owns duplicate target diagnostics and seen-target
insertion, `layout/supported_platforms.rs` owns supported platform traversal, and
`layout/supported_platforms/state.rs` owns seen-platform state creation, and
`layout/supported_platforms/uniqueness.rs` owns duplicate platform diagnostics and seen-platform
insertion before catalog availability or export planning consumes the package layout metadata.

Static manifest package-layout tests mirror that split. `static_manifest_contracts/package_layout.rs`
is child-module wiring only, `package_layout/arrays.rs` owns string-array parsing and duplicate
entry diagnostics, `package_layout/default_packaging.rs` owns package and optional-feature export
strategy rows, `package_layout/roots.rs` owns `asset_roots` / `content_roots` relative-path checks,
and `package_layout/supported_platforms.rs` owns static export-platform allow-list coverage. This
keeps static plugin TOML layout checks aligned with production package-layout validation without
changing public test names or manifest behavior.

Package coordinate validation is folder-backed under `package_validation/coordinates.rs`. The owner
keeps the internal entry point, `coordinates/fields.rs` owns coordinate presence gating and shape
dispatch, `coordinates/presence.rs` owns presence orchestration,
`coordinates/presence/fields.rs` owns borrowed coordinate field declaration state,
`coordinates/presence/completeness.rs` owns the all-or-none `package_prefix` / `package_company` /
`package_name` completeness diagnostic, `coordinates/shape.rs` owns coordinate shape dispatch,
`coordinates/shape/prefix.rs` owns reverse-DNS prefix segment validation, and
`coordinates/shape/segment.rs` owns company/name lowercase segment diagnostics.

Static manifest package-coordinate tests now follow the same helper ownership. `static_manifest_contracts/package_coordinates.rs`
is child-module wiring only, `package_coordinates/helpers.rs` owns coordinate-field presence and
resolved package-id construction, `package_coordinates/coordinates.rs` owns coordinate field shape
checks, and `package_coordinates/resolved_ids.rs` owns global resolved-package-id uniqueness. This
keeps coordinate parsing separate from row-level assertions before catalog identity and native
package loading consume package coordinates.

Static manifest package-metadata tests are folder-backed around the same public metadata surface.
`static_manifest_contracts/package_metadata.rs` is child-module wiring only,
`package_metadata/public_fields.rs` owns required public package fields,
`package_metadata/display.rs` owns trimmed display/description/note text checks,
`package_metadata/classification.rs` owns category and maturity allow-lists,
`package_metadata/targets.rs` owns supported target and module target-mode allow-lists, and
`package_metadata/arrays.rs` owns duplicate string-array checks for package, feature, and module
metadata rows. This keeps static public metadata validation split by field family before catalog,
profile, and export projections consume those rows.

Package root validation follows the same shape under `package_validation/roots.rs`. The owner keeps
the `asset_roots` and `content_roots` entry point, `roots/array.rs` owns per-array row traversal
and path validation dispatch, `roots/array/state.rs` owns seen-root state creation,
`roots/array/uniqueness.rs` owns duplicate root diagnostics and seen-root insertion,
`roots/path.rs` owns root path rule dispatch, `roots/path/field.rs` owns non-empty/trimmed
diagnostics, `roots/path/relative.rs` owns relative-path diagnostics, `roots/path/separator.rs`
owns forward-slash diagnostics, and `roots/path/segments.rs` owns empty/current/parent segment
diagnostics.

Package module validation is folder-backed over the shared module-validation helpers.
`package_validation/modules.rs` keeps the internal entry point, `modules/field.rs` adapts package
field diagnostics to the shared module validators, `modules/rows.rs` owns module list traversal and
`modules/rows/state.rs` owns seen-name state creation, `modules/row.rs` owns per-module dispatch order, `modules/row/identity.rs` owns
identity rule dispatch, `modules/row/identity/name.rs` owns package-id owner checks,
`modules/row/identity/crate_name.rs` owns module crate-name checks, `modules/row/capabilities.rs` owns the
package namespace adapter for shared module capability checks, and `modules/row/target_modes.rs`
owns package supported-target coverage dispatch. This keeps package-specific module traversal
separate from shared runtime/editor module rule implementations.

Package contribution validation is folder-backed around a small owner. `package_validation/contributions.rs`
keeps the internal entry point, while `contributions/groups.rs` owns manifest-local duplicate and owner
validation group dispatch in the same order registration reports consume it.

Manifest-local contribution duplicate validation is also folder-backed. The
`package_validation/contribution_duplicates.rs` owner keeps the entry points consumed by
`contributions.rs`, while `contribution_duplicates/options.rs` owns option row traversal,
`contribution_duplicates/options/state.rs` owns seen-key state creation and lifetime typing,
`contribution_duplicates/options/uniqueness.rs` owns duplicate option-key diagnostics plus seen
insertion, `contribution_duplicates/event_catalogs.rs` owns event catalog
row traversal, `contribution_duplicates/event_catalogs/state.rs` owns seen-namespace state creation,
`contribution_duplicates/event_catalogs/uniqueness.rs` owns duplicate event-catalog namespace diagnostics plus seen insertion, `contribution_duplicates/components.rs`
owns component row traversal, `contribution_duplicates/components/state.rs` owns seen-type state creation,
`contribution_duplicates/components/uniqueness.rs`
owns duplicate component type-id diagnostics plus seen insertion, `contribution_duplicates/ui_components.rs`
owns UI component row traversal, `contribution_duplicates/ui_components/state.rs` owns seen-id state creation,
and `contribution_duplicates/ui_components/uniqueness.rs` owns duplicate UI component id diagnostics
plus seen insertion. This keeps duplicate-row diagnostics separate from contribution package-owner
checks and registry installation.

Contribution package-owner validation uses the same owner/child split.
`package_validation/contribution_owners.rs` keeps the entry points consumed by `contributions.rs`,
while `contribution_owners/event_catalogs.rs` owns event catalog row traversal,
`contribution_owners/event_catalogs/prefix.rs` owns expected package-prefix construction,
`contribution_owners/event_catalogs/ownership.rs` owns event-catalog namespace package-prefix diagnostics,
`contribution_owners/components.rs` owns component row traversal,
`contribution_owners/components/ownership.rs` owns component `plugin_id` ownership diagnostics,
`contribution_owners/ui_components.rs` owns UI component row traversal, and
`contribution_owners/ui_components/ownership.rs` owns UI component `plugin_id` ownership diagnostics.
This keeps manifest ownership diagnostics separate from duplicate-row scans and registry installation.

External optional-feature providers are resolved during export planning from the completed project
plugin manifest. Enabled owner selections contribute external feature packages only when the feature
is enabled, target-compatible, and carries a provider package id that differs from the owner plugin.
This prevents disabled catalog defaults from leaking extra native or linked feature packages into a
desktop export plan.

Runtime module manifests now carry scheduler declarations for plugin architecture v2.
`PluginModuleManifest.system_sets` declares the dot-namespaced `SystemSet` names a module owns, and
`PluginModuleManifest.system_anchors` declares the stable system ids that other plugins may target
with before/after constraints. These fields are serde-defaulted for existing manifests and are set by
`with_system_sets(...)` / `with_system_anchors(...)` on both module builders and
`RuntimePluginDescriptor`. Descriptor-owned values project into the generated `.runtime` module row
so linked Rust plugins and static package manifests share the same public scheduler contract.

Package validation treats system sets and system anchors as module-owned namespace declarations.
Each value must be non-empty, lowercase dot-namespaced, prefixed by the package id, and unique inside
the declaring module row. Registration reports then check the dynamic side of the contract: every
declared runtime module `system_anchor` must be registered as a real ECS system by the same interned
plugin module owner. A system with the same id registered by a different module does not satisfy the
declaring module's anchor. This keeps catalog/export/editor contracts aligned with unloadable,
owner-tracked runtime registration and avoids using manifest-only placeholders as scheduling anchors.

## Constraints

- Option keys must be non-empty and trimmed.
- Option keys and option capabilities must use at least two lowercase dot-separated namespace segments.
- Option value types must be one of `bool`, `integer`, `number`, `string`, or `enum`.
- Boolean, integer, and number defaults must parse as their declared type; number defaults must be finite.
- Enum options must declare unique `enum_values` and the default must be present in that list.
- Event catalog namespaces must be non-empty, trimmed, lowercase, and dot-namespaced.
- Event catalogs must have a positive version and at least one event.
- Event ids must be namespace-prefixed by their catalog namespace.
- Event payload schemas must stay under the package namespace and end with a positive `vN` segment when present.
- Duplicate option keys and event namespaces are rejected by the runtime extension registry.
- Asset importer descriptors must declare at least one source extension or full suffix before they can be registered as diagnostic-only manifest declarations.
- Duplicate importer ids and duplicate importer matchers at the same priority are rejected by the asset importer registry.
- Asset importer descriptors cannot register `.ui.toml` or `.v2.ui.toml`; UI component importers must target `.zui` on the production path.
- UI component descriptors must reference `.zui` documents; legacy `.ui.toml` and `.v2.ui.toml` are reserved for migration and fixture tests.
- Runtime module `system_sets` and `system_anchors` must use the package id as their namespace prefix and must be unique within the module row.
- Runtime module `system_anchors` are accepted only when the same runtime module owner registers a matching ECS system id during runtime extension registration.
- `provides_interfaces` rows must declare unique, non-empty, trimmed, lowercase dot-namespaced interface ids.
- Dependency `interfaces` entries must be unique within that dependency row and use the same lowercase dot-namespace shape.
- A dependency row must declare a capability, at least one interface, or both.
- Existing plugin manifests continue to deserialize because the new fields use serde defaults.
- This layer records declared dependency metadata; `RuntimePluginCatalog` resolves required bridge interface dependency closure after registration reports are merged. Required dependency rows with interface ids become blocking `bridge.strong_dependency_missing` diagnostics when the provider package is absent or does not declare the requested interface. Optional interface dependency rows remain non-blocking. The same required rows drive `RuntimePluginCatalog::strong_bridge_dependents(...)` and `strong_bridge_disable_blockers(...)`, which list dependents for future strong-target disable rejection.

## Test Coverage

The sound plugin registration test proves a real package can contribute dependencies, options, components, and a concrete event catalog through both its manifest and runtime extension registry.

The independent plugin follow-up adds focused runtime coverage proving `RuntimePluginRegistrationReport::from_plugin(...)` collects manifest-declared options, event catalogs, component descriptors, UI component descriptors, and asset importer descriptors, and that `RuntimePluginCatalog::runtime_extensions()` preserves those contributions when merging registration reports.

The review follow-up adds package-manifest coverage for overriding `default_packaging` through the builder API and validates the plugin workspace with `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --jobs 1`.

The workspace-shape plugin contract now also checks `RuntimePluginDescriptor::builtin_catalog()` optional-feature rows against each first-party static `plugin.toml` manifest. That guard catches catalog drift for Sound, Net, Particles, Rendering, and future owner-embedded feature bundles before profile, export, or editor status code consumes divergent feature metadata.

The plugin architecture follow-up extends `RuntimePluginDescriptor` projection coverage so descriptor
`system_sets` and `system_anchors` appear on the generated runtime module row. It also adds
registration-report coverage for missing anchors, anchors registered by the wrong module owner, and
anchors registered by the declaring module owner. Native/static package-manifest coverage now rejects
cross-package system-set prefixes, duplicate system-set declarations, malformed system-anchor names,
and duplicate system-anchor declarations. `rustfmt --check` and `git diff --check` passed for the
touched Rust files. The focused runtime command `cargo test -p zircon_runtime --lib
plugin_extensions::runtime_plugin_descriptor --locked --jobs 1 --target-dir
D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never --
--nocapture` was attempted on 2026-06-12 and timed out after 10 minutes under concurrent runtime
Cargo load; no pass is claimed for that command yet.

The bridge-manifest follow-up adds `PluginInterfaceManifest`, top-level `provides_interfaces`,
dependency-level `interfaces`, package validation for interface namespace and uniqueness, and static
plugin schema awareness for the new TOML fields. `package_manifest_declarations.rs` covers
builder/TOML roundtrip, `runtime_plugin_package_manifest.rs` covers malformed bridge interface rows
and interface-only dependency rows, and `static_manifest_contracts/interfaces.rs` covers static
plugin TOML interface namespace shape. `cargo check -p zircon_runtime --lib --locked --jobs 1
--target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short
--color never` passed with existing warnings after this slice. `cargo test -p zircon_runtime
--lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir
D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never --
--test-threads=1` passed 32 package-manifest validation tests, including the new bridge interface
diagnostics and interface-only dependency row. `cargo test -q -p zircon_runtime --lib
plugin_package_manifest_declares_bridge_interfaces --locked --jobs 1 --target-dir
D:\cargo-targets\zircon-plugin-architecture-bridge-0612 -- --test-threads=1` passed the focused
builder/TOML roundtrip test. `cargo test -q -p zircon_runtime --lib
plugin_tomls_declare_bridge_interface_namespaces --locked --jobs 1 --target-dir
D:\cargo-targets\zircon-plugin-architecture-bridge-0612 -- --test-threads=1` was attempted but is
currently blocked before the static contract can run by an unrelated lib-test compile error in
`zircon_runtime/src/ui/tests/asset_resource_resolver.rs:225` (`&str` passed where `String` is
required).

The bridge export-consistency follow-up adds linked Rust registration diagnostics for
declared-but-unexported and exported-but-undeclared interface ids. Focused tests were added to
`runtime_plugin_package_manifest.rs`, but fresh execution is currently blocked before those tests
can run by unrelated UI lib-test compile failures: `zircon_runtime/src/ui/tests/component_catalog/component_state.rs:9`
references a missing `component_state/button.rs`, and the latest library check is blocked by
`zircon_runtime/src/ui/component/state_reducer/button.rs:8` because `UiComponentEvent` does not
implement `Eq`.

The bridge dependency-closure follow-up adds catalog-level enforcement for required interface
dependencies declared in package manifests. `runtime_plugin_bridge_dependencies.rs` covers missing
providers, present providers, optional missing providers, and transitive dependency-chain
diagnostics. `rustfmt` passed for the touched catalog/test files. Focused execution of
`cargo test -p zircon_runtime --lib runtime_plugin_bridge_dependencies --locked --jobs 1 --target-dir
D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never --
--test-threads=1` was attempted twice on 2026-06-12 but timed out during lib-test compilation while
unrelated UI/render Cargo jobs were active; no pass is claimed for this new slice yet.

`cargo check -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short` and `cargo test -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short` now pass from the `zircon_plugins` workspace using `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-closeout`. The sound test run covered one editor registration test and seven runtime mixer/DSP/manifest tests.

2026-06-01 M6 plugin-workspace validation exercised these stricter registry checks across first-party plugins. Navigation, Net, Particles, and Sound had stale metadata or stale assertions corrected so the full plugin workspace command `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose` could complete. The fixes were in plugin package manifests and tests: enum options now declare values, string options have non-empty defaults, event namespaces and payload schemas are package-prefixed/versioned, and old placeholder event-catalog expectations were removed.

2026-06-03 scoped runtime validation rechecked the package-manifest and registration-report code
after the M6 package helper visibility fixes and adjacent render-feature export wiring:
`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir
E:\cargo-targets\zircon-runtime-plugin-asset-importer-metadata-subgroups` passed with existing
runtime warnings only. The log is
`.codex/tmp/asset_m6_runtime_check_after_editor_visible_frame_split_20260603.log`; it is a scoped
library type-check and does not replace the full plugin/workspace gates above.
