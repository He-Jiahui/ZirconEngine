use std::collections::BTreeMap;

use super::super::{for_each_static_plugin_manifest, non_empty_string_value, optional_table_array};
use super::properties::validate_component_properties;
use super::shape::{assert_lowercase_dot_namespace, assert_prefixed_by_package, assert_trimmed};
use super::uniqueness::assert_unique_row;

#[test]
fn plugin_tomls_declare_component_rows() {
    let mut component_type_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let Some(components) =
            optional_table_array(table, relative_path, "top-level", "components")
        else {
            return;
        };
        assert!(
            !components.is_empty(),
            "plugin manifest {relative_path:?} components should not be empty when declared"
        );

        for component in components {
            let type_id = non_empty_string_value(component, relative_path, "component", "type_id");
            let component_context = format!("component `{type_id}`");
            assert_lowercase_dot_namespace(relative_path, &component_context, "type_id", type_id);
            assert_prefixed_by_package(
                relative_path,
                &component_context,
                "type_id",
                type_id,
                package_id,
            );
            assert_unique_row(
                relative_path,
                &mut component_type_ids,
                type_id,
                component_context.clone(),
            );

            let plugin_id =
                non_empty_string_value(component, relative_path, &component_context, "plugin_id");
            assert_eq!(
                plugin_id, package_id,
                "plugin manifest {relative_path:?} {component_context} plugin_id `{plugin_id}` should match package id `{package_id}`"
            );

            let display_name = non_empty_string_value(
                component,
                relative_path,
                &component_context,
                "display_name",
            );
            assert_trimmed(
                relative_path,
                &component_context,
                "display_name",
                display_name,
            );

            validate_component_properties(relative_path, component, &component_context);
        }
    });
}
