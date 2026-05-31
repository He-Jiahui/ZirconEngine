use std::path::Path;

use super::for_each_static_plugin_manifest;

const KNOWN_TOP_LEVEL_FIELDS: [&str; 23] = [
    "asset_importers",
    "asset_roots",
    "capabilities",
    "capability_statuses",
    "category",
    "content_roots",
    "default_packaging",
    "dependencies",
    "description",
    "display_name",
    "event_catalogs",
    "id",
    "maturity",
    "modules",
    "optional_features",
    "options",
    "package_company",
    "package_name",
    "package_prefix",
    "sdk_api_version",
    "supported_platforms",
    "supported_targets",
    "version",
];
const KNOWN_ASSET_IMPORTER_FIELDS: [&str; 9] = [
    "additional_output_kinds",
    "full_suffixes",
    "id",
    "importer_version",
    "output_kind",
    "plugin_id",
    "priority",
    "required_capabilities",
    "source_extensions",
];
const KNOWN_CAPABILITY_STATUS_FIELDS: [&str; 5] = [
    "bevy_references",
    "capability",
    "note",
    "status",
    "target_modes",
];
const KNOWN_DEPENDENCY_FIELDS: [&str; 3] = ["capability", "id", "required"];
const KNOWN_EVENT_CATALOG_FIELDS: [&str; 3] = ["events", "namespace", "version"];
const KNOWN_EVENT_FIELDS: [&str; 3] = ["display_name", "id", "payload_schema"];
const KNOWN_MODULE_FIELDS: [&str; 5] =
    ["capabilities", "crate_name", "kind", "name", "target_modes"];
const KNOWN_OPTION_FIELDS: [&str; 5] = [
    "default_value",
    "display_name",
    "key",
    "required_capability",
    "value_type",
];
const KNOWN_OPTIONAL_FEATURE_FIELDS: [&str; 8] = [
    "capabilities",
    "default_packaging",
    "dependencies",
    "display_name",
    "enabled_by_default",
    "id",
    "modules",
    "owner_plugin_id",
];
const KNOWN_OPTIONAL_FEATURE_DEPENDENCY_FIELDS: [&str; 3] = ["capability", "plugin_id", "primary"];

#[test]
fn plugin_tomls_declare_known_top_level_fields() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in table.keys() {
            assert!(
                KNOWN_TOP_LEVEL_FIELDS.contains(&field_name.as_str()),
                "plugin manifest {relative_path:?} top-level field `{field_name}` is not a known PluginPackageManifest field"
            );
        }
    });
}

#[test]
fn plugin_tomls_declare_known_nested_fields() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_row_fields(
            table,
            relative_path,
            "asset_importers",
            "asset importer",
            &KNOWN_ASSET_IMPORTER_FIELDS,
        );
        assert_known_row_fields(
            table,
            relative_path,
            "capability_statuses",
            "capability status",
            &KNOWN_CAPABILITY_STATUS_FIELDS,
        );
        assert_known_row_fields(
            table,
            relative_path,
            "dependencies",
            "top-level dependency",
            &KNOWN_DEPENDENCY_FIELDS,
        );
        assert_known_row_fields(
            table,
            relative_path,
            "modules",
            "package module",
            &KNOWN_MODULE_FIELDS,
        );
        assert_known_row_fields(
            table,
            relative_path,
            "options",
            "option",
            &KNOWN_OPTION_FIELDS,
        );

        for feature in optional_table_array(table, relative_path, "top-level", "optional_features")
        {
            let feature_id = feature
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let feature_context = format!("optional feature `{feature_id}`");
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

        for catalog in optional_table_array(table, relative_path, "top-level", "event_catalogs") {
            let namespace = catalog
                .get("namespace")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let catalog_context = format!("event catalog `{namespace}`");
            assert_known_table_fields(
                relative_path,
                &catalog_context,
                catalog,
                &KNOWN_EVENT_CATALOG_FIELDS,
            );
            assert_known_row_fields(
                catalog,
                relative_path,
                "events",
                &format!("{catalog_context} event"),
                &KNOWN_EVENT_FIELDS,
            );
        }
    });
}

fn assert_known_row_fields(
    table: &toml::Table,
    relative_path: &Path,
    field_name: &str,
    row_context: &str,
    known_fields: &[&str],
) {
    for row in optional_table_array(table, relative_path, row_context, field_name) {
        assert_known_table_fields(relative_path, row_context, row, known_fields);
    }
}

fn optional_table_array<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> Vec<&'a toml::Table> {
    let Some(value) = table.get(field_name) else {
        return Vec::new();
    };
    value
        .as_array()
        .unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} {context} `{field_name}` should be an array")
        })
        .iter()
        .map(|row| {
            row.as_table().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {context} `{field_name}` row should be a table"
                )
            })
        })
        .collect()
}

fn assert_known_table_fields(
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
