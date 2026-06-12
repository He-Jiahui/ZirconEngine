mod asset_importers;
mod capabilities;
mod components;
mod dependencies;
mod dependency_rows;
mod event_catalogs;
mod feature_bundles;
mod feature_extensions;
mod interfaces;
mod manifest_schema;
mod modules;
mod optional_features;
mod options;
mod package_coordinates;
mod package_identity;
mod package_kind;
mod package_layout;
mod package_metadata;
mod package_versions;
mod support;
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
