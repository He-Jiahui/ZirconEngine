use std::collections::BTreeMap;
use std::path::Path;

use super::super::{
    assert_known_runtime_targets, assert_non_empty_string, assert_unique_string_array_entries,
    for_each_static_plugin_manifest, non_empty_string_value,
};
use super::traversal::capability_status_array;

#[test]
fn plugin_tomls_declare_capability_status_rows() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(statuses) = capability_status_array(table, relative_path) else {
            return;
        };
        assert!(
            !statuses.is_empty(),
            "plugin manifest {relative_path:?} capability_statuses should not be empty when declared"
        );

        let mut capability_statuses = BTreeMap::new();
        for status in statuses {
            let status = status.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} capability status should be a table")
            });
            let capability =
                non_empty_string_value(status, relative_path, "capability status", "capability");
            let status_context = format!("capability status `{capability}`");
            if let Some(previous_context) =
                capability_statuses.insert(capability.to_string(), status_context.clone())
            {
                panic!(
                    "plugin manifest {relative_path:?} capability status `{capability}` should be unique; first declared by {previous_context}, repeated by {status_context}"
                );
            }

            assert_known_capability_status(status, relative_path, &status_context);
            if status.get("target_modes").is_some() {
                assert_known_runtime_targets(
                    status,
                    relative_path,
                    &status_context,
                    "target_modes",
                );
                assert_unique_string_array_entries(
                    status,
                    relative_path,
                    &status_context,
                    "target_modes",
                );
            }
            assert_optional_unique_string_array_entries(
                status,
                relative_path,
                &status_context,
                "bevy_references",
            );
            if status.get("note").is_some() {
                assert_non_empty_string(status, relative_path, &status_context, "note");
            }
        }
    });
}

fn assert_optional_unique_string_array_entries(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    if table.get(field_name).is_some() {
        assert_unique_string_array_entries(table, relative_path, context, field_name);
    }
}

fn assert_known_capability_status(table: &toml::Table, relative_path: &Path, context: &str) {
    let status = non_empty_string_value(table, relative_path, context, "status");
    assert!(
        matches!(
            status,
            "complete" | "partial" | "stub" | "externalized" | "unsupported"
        ),
        "plugin manifest {relative_path:?} {context} status `{status}` should be a known capability status"
    );
}
