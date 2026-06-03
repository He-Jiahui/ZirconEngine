use super::{
    assert_known_default_packaging_strategies, assert_known_runtime_targets,
    assert_unique_dependency_row, assert_unique_string_array_entries, bool_value,
    for_each_static_plugin_manifest, non_empty_string_array_values, non_empty_string_value,
    optional_table_array, required_table_array,
};

mod capabilities;
mod dependencies;
mod metadata;
mod modules;
mod shape;
mod traversal;
mod uniqueness;

use capabilities::static_package_capabilities;
use shape::{
    assert_crate_name_shape, assert_lowercase_dot_namespace, assert_package_token, assert_trimmed,
};
use traversal::visit_feature_extension_rows;
use uniqueness::{assert_unique_identity, assert_unique_provider_row};
