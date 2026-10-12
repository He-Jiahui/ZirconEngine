use super::super::{for_each_static_plugin_manifest, non_empty_string_value};
use super::semver::assert_semver_core;

#[test]
fn plugin_tomls_declare_semantic_package_versions() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in ["version", "sdk_api_version"] {
            let value = non_empty_string_value(table, relative_path, "top-level", field_name);
            assert_semver_core(relative_path, "top-level", field_name, value);
        }
    });
}
