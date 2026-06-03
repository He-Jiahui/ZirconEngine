use std::path::Path;

use super::super::optional_table_array;
use super::field_sets::{
    KNOWN_COMPONENT_FIELDS, KNOWN_COMPONENT_PROPERTY_FIELDS, KNOWN_MODULE_FIELDS,
    KNOWN_OPTIONAL_FEATURE_DEPENDENCY_FIELDS, KNOWN_OPTIONAL_FEATURE_FIELDS,
};

pub(super) fn assert_known_component_fields(table: &toml::Table, relative_path: &Path) {
    for component in
        optional_table_array(table, relative_path, "top-level", "components").unwrap_or_default()
    {
        let type_id = component
            .get("type_id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let component_context = format!("component `{type_id}`");
        assert_known_table_fields(
            relative_path,
            &component_context,
            component,
            &KNOWN_COMPONENT_FIELDS,
        );
        assert_known_row_fields(
            component,
            relative_path,
            "properties",
            &format!("{component_context} property"),
            &KNOWN_COMPONENT_PROPERTY_FIELDS,
        );
    }
}

pub(super) fn assert_known_feature_bundle_fields(
    table: &toml::Table,
    relative_path: &Path,
    field_name: &str,
    row_context: &str,
) {
    for feature in
        optional_table_array(table, relative_path, "top-level", field_name).unwrap_or_default()
    {
        let feature_id = feature
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let feature_context = format!("{row_context} `{feature_id}`");
        assert_known_table_fields(
            relative_path,
            &feature_context,
            feature,
            &KNOWN_OPTIONAL_FEATURE_FIELDS,
        );
        assert_known_row_fields(
            feature,
            relative_path,
            "dependencies",
            &format!("{feature_context} dependency"),
            &KNOWN_OPTIONAL_FEATURE_DEPENDENCY_FIELDS,
        );
        assert_known_row_fields(
            feature,
            relative_path,
            "modules",
            &format!("{feature_context} module"),
            &KNOWN_MODULE_FIELDS,
        );
    }
}

pub(super) fn assert_known_row_fields(
    table: &toml::Table,
    relative_path: &Path,
    field_name: &str,
    row_context: &str,
    known_fields: &[&str],
) {
    for row in
        optional_table_array(table, relative_path, row_context, field_name).unwrap_or_default()
    {
        assert_known_table_fields(relative_path, row_context, row, known_fields);
    }
}

pub(super) fn assert_known_table_fields(
    relative_path: &Path,
    context: &str,
    table: &toml::Table,
    known_fields: &[&str],
) {
    for field_name in table.keys() {
        assert!(
            known_fields.contains(&field_name.as_str()),
            "plugin manifest {relative_path:?} {context} field `{field_name}` is not a known PluginPackageManifest field"
        );
    }
}
