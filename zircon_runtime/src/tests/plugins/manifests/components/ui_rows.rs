use std::collections::BTreeMap;

use super::super::{for_each_static_plugin_manifest, non_empty_string_value, optional_table_array};
use super::shape::{
    assert_lowercase_dot_namespace, assert_package_path, assert_prefixed_by_package, assert_trimmed,
};
use super::uniqueness::assert_unique_row;

#[test]
fn plugin_tomls_declare_ui_component_rows() {
    let mut ui_component_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let Some(ui_components) =
            optional_table_array(table, relative_path, "top-level", "ui_components")
        else {
            return;
        };
        assert!(
            !ui_components.is_empty(),
            "plugin manifest {relative_path:?} ui_components should not be empty when declared"
        );

        for component in ui_components {
            let component_id =
                non_empty_string_value(component, relative_path, "ui component", "component_id");
            let component_context = format!("ui component `{component_id}`");
            assert_lowercase_dot_namespace(
                relative_path,
                &component_context,
                "component_id",
                component_id,
            );
            assert_prefixed_by_package(
                relative_path,
                &component_context,
                "component_id",
                component_id,
                package_id,
            );
            assert_unique_row(
                relative_path,
                &mut ui_component_ids,
                component_id,
                component_context.clone(),
            );

            let plugin_id =
                non_empty_string_value(component, relative_path, &component_context, "plugin_id");
            assert_eq!(
                plugin_id, package_id,
                "plugin manifest {relative_path:?} {component_context} plugin_id `{plugin_id}` should match package id `{package_id}`"
            );

            let ui_document =
                non_empty_string_value(component, relative_path, &component_context, "ui_document");
            assert_trimmed(
                relative_path,
                &component_context,
                "ui_document",
                ui_document,
            );
            assert!(
                ui_document.ends_with(".zui"),
                "plugin manifest {relative_path:?} {component_context} ui_document `{ui_document}` should reference a .zui component asset"
            );
            assert_package_path(
                relative_path,
                &component_context,
                "ui_document",
                ui_document,
            );
        }
    });
}
