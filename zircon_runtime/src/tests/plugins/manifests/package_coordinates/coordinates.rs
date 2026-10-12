use std::path::Path;

use super::super::{for_each_static_plugin_manifest, non_empty_string_value};
use super::package_coordinate_resolution::declares_any_coordinate_field;

#[test]
fn plugin_tomls_declare_package_coordinates() {
    for_each_static_plugin_manifest(|relative_path, table| {
        if !declares_any_coordinate_field(table) {
            return;
        }

        let package_prefix = non_empty_string_value(
            table,
            relative_path,
            "package coordinates",
            "package_prefix",
        );
        let package_company = non_empty_string_value(
            table,
            relative_path,
            "package coordinates",
            "package_company",
        );
        let package_name =
            non_empty_string_value(table, relative_path, "package coordinates", "package_name");

        assert_package_prefix(relative_path, package_prefix);
        assert_coordinate_segment(relative_path, "package_company", package_company);
        assert_coordinate_segment(relative_path, "package_name", package_name);
    });
}

fn assert_package_prefix(relative_path: &Path, package_prefix: &str) {
    assert_trimmed(relative_path, "package_prefix", package_prefix);
    assert!(
        package_prefix.split('.').all(is_valid_coordinate_segment),
        "plugin manifest {relative_path:?} package_prefix `{package_prefix}` should contain only non-empty lowercase coordinate segments"
    );
}

fn assert_coordinate_segment(relative_path: &Path, field_name: &str, segment: &str) {
    assert_trimmed(relative_path, field_name, segment);
    assert!(
        is_valid_coordinate_segment(segment),
        "plugin manifest {relative_path:?} {field_name} `{segment}` should be a non-empty lowercase coordinate segment"
    );
}

fn assert_trimmed(relative_path: &Path, field_name: &str, value: &str) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {field_name} `{value}` should not have leading or trailing whitespace"
    );
}

fn is_valid_coordinate_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
