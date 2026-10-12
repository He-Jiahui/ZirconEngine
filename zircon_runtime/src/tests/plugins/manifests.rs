//! 静态插件包清单的领域回归入口；共享读取与遍历只服务清单契约，包声明、模块布局和实际运行登记分别验证。
#[path = "manifests/asset_importers.rs"]
mod asset_importers;
#[path = "manifests/capabilities.rs"]
mod capabilities;
#[path = "manifests/components.rs"]
mod components;
#[path = "manifests/dependencies.rs"]
mod dependencies;
#[path = "manifests/dependency_rows.rs"]
mod dependency_rows;
#[path = "manifests/event_catalogs.rs"]
mod event_catalogs;
#[path = "manifests/feature_bundles.rs"]
mod feature_bundles;
#[path = "manifests/feature_extensions.rs"]
mod feature_extensions;
#[path = "manifests/interfaces.rs"]
mod interfaces;
#[path = "manifests/manifest_schema.rs"]
mod manifest_schema;
#[path = "manifests/modules.rs"]
mod modules;
#[path = "manifests/optional_features.rs"]
mod optional_features;
#[path = "manifests/options.rs"]
mod options;
#[path = "manifests/package_coordinates.rs"]
mod package_coordinates;
#[path = "manifests/package_identity.rs"]
mod package_identity;
#[path = "manifests/package_kind.rs"]
mod package_kind;
#[path = "manifests/package_layout.rs"]
mod package_layout;
#[path = "manifests/package_metadata.rs"]
mod package_metadata;
#[path = "manifests/package_versions.rs"]
mod package_versions;
#[path = "manifests/support.rs"]
mod support;
#[path = "manifests/table_rows.rs"]
mod table_rows;

use dependency_rows::{
    visit_asset_importer_required_capabilities, visit_feature_dependency_rows,
    visit_option_required_capabilities, visit_package_dependency_ids,
    visit_package_dependency_rows,
};
use feature_bundles::{
    for_each_feature_extension, for_each_module_row, for_each_optional_feature, visit_module_rows,
};
use support::{
    assert_known_default_packaging_strategies, assert_known_runtime_targets,
    assert_non_empty_string, assert_non_empty_string_array, assert_unique_dependency_row,
    assert_unique_static_identity, assert_unique_string_array_entries, bool_value,
    for_each_static_plugin_manifest, integer_value, non_empty_string_array_values,
    non_empty_string_value, plugins_workspace_root,
};
use table_rows::{optional_table_array, required_table_array};
