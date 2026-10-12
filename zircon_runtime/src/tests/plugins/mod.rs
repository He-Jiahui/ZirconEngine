//! 插件整合回归入口；元数据、扩展登记、目录选择、导出模板和桥生命周期各有明确测试所有者，静态清单约束由独立子树检查。
#[path = "asset_importer_install.rs"]
mod asset_importer_install;
#[path = "builtin_catalog_features.rs"]
mod builtin_catalog_features;
#[path = "dynamic_components.rs"]
mod dynamic_components;
#[path = "export_build_plan.rs"]
mod export_build_plan;
#[path = "export_build_plan_feature_provider.rs"]
mod export_build_plan_feature_provider;
#[path = "export_build_plan_native_dynamic.rs"]
mod export_build_plan_native_dynamic;
#[path = "export_build_plan_platform.rs"]
mod export_build_plan_platform;
#[path = "export_build_plan_profile.rs"]
mod export_build_plan_profile;
#[path = "export_build_plan_provider_package.rs"]
mod export_build_plan_provider_package;
#[cfg(feature = "graphics")]
#[path = "extension_registry.rs"]
mod extension_registry;
#[path = "extension_registry_bridge.rs"]
mod extension_registry_bridge;
#[path = "extension_registry_bridge_perf_baseline.rs"]
mod extension_registry_bridge_performance_baseline;
#[cfg(feature = "ui")]
#[path = "extension_registry_components.rs"]
mod extension_registry_components;
#[path = "extension_registry_event_catalogs.rs"]
mod extension_registry_event_catalogs;
#[path = "extension_registry_features.rs"]
mod extension_registry_features;
#[path = "extension_registry_managers.rs"]
mod extension_registry_managers;
#[cfg(all(feature = "graphics", feature = "ui"))]
#[path = "extension_registry_metadata.rs"]
mod extension_registry_metadata;
#[path = "extension_registry_modules.rs"]
mod extension_registry_modules;
#[path = "extension_registry_options.rs"]
mod extension_registry_options;
#[path = "extension_registry_systems.rs"]
mod extension_registry_systems;
#[cfg(feature = "graphics")]
#[path = "extension_registry_typed_points.rs"]
mod extension_registry_typed_points;
#[path = "manifest_contributions.rs"]
mod manifest_contributions;
#[path = "loader.rs"]
mod native_plugin_loader;
#[path = "package_manifest_declarations.rs"]
mod package_manifest_declarations;
#[path = "package_manifest_options.rs"]
mod package_manifest_options;
#[path = "plugin_workspace_shape.rs"]
mod plugin_workspace_shape;
#[path = "profile_availability_project.rs"]
mod profile_availability_projection;
#[path = "profile_maturity.rs"]
mod profile_maturity;
#[path = "project_plugin_manifest.rs"]
mod project_plugin_manifest;
#[path = "runtime_plugin_bridge_dependencies.rs"]
mod runtime_plugin_bridge_dependencies;
#[path = "runtime_plugin_catalog_features.rs"]
mod runtime_plugin_catalog_features;
#[path = "runtime_plugin_desc.rs"]
mod runtime_plugin_descriptor;
#[path = "runtime_plugin_feature_desc.rs"]
mod runtime_plugin_feature_descriptor;
#[path = "runtime_plugin_lifecycle.rs"]
mod runtime_plugin_lifecycle;
#[path = "runtime_plugin_package_manifest.rs"]
mod runtime_plugin_package_manifest;
#[path = "manifests.rs"]
mod static_manifest_contracts;
