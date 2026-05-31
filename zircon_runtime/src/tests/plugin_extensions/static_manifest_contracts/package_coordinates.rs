use std::collections::BTreeMap;
use std::path::Path;

use super::{for_each_static_plugin_manifest, non_empty_string_value};

const COORDINATE_FIELDS: [&str; 3] = ["package_prefix", "package_company", "package_name"];

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

#[test]
fn plugin_tomls_declare_unique_resolved_package_ids() {
    let mut resolved_package_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let resolved_package_id = resolved_package_id(table, relative_path);
        if let Some(previous_path) = resolved_package_ids.insert(
            resolved_package_id.clone(),
            relative_path.display().to_string(),
        ) {
            panic!(
                "resolved package id `{resolved_package_id}` should be globally unique; first declared by {previous_path}, repeated by {}",
                relative_path.display()
            );
        }
    });
}

fn declares_any_coordinate_field(table: &toml::Table) -> bool {
    COORDINATE_FIELDS
        .iter()
        .any(|field_name| table.get(*field_name).is_some())
}

fn resolved_package_id(table: &toml::Table, relative_path: &Path) -> String {
    if declares_any_coordinate_field(table) {
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
        return format!("{package_prefix}.{package_company}.{package_name}");
    }

    non_empty_string_value(table, relative_path, "top-level", "id").to_string()
}

fn assert_package_prefix(relative_path: &Path, package_prefix: &str) {
    assert_trimmed(relative_path, "package_prefix", package_prefix);
    assert!(
        package_prefix
            .split('.')
            .all(is_valid_coordinate_segment),
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
