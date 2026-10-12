use std::collections::BTreeMap;

use super::{
    assert_dot_namespaced_event_id, assert_event_rows, event_catalog_array,
    for_each_static_plugin_manifest, integer_value, non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_event_catalog_rows() {
    let mut namespaces = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(catalogs) = event_catalog_array(table, relative_path) else {
            return;
        };
        assert!(
            !catalogs.is_empty(),
            "plugin manifest {relative_path:?} event_catalogs should not be empty when declared"
        );

        for catalog in catalogs {
            let catalog = catalog.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} event catalog should be a table")
            });
            let namespace =
                non_empty_string_value(catalog, relative_path, "event catalog", "namespace");
            let catalog_context = format!("event catalog `{namespace}`");
            assert_dot_namespaced_event_id(relative_path, &catalog_context, "namespace", namespace);
            if let Some(previous_context) =
                namespaces.insert(namespace.to_string(), catalog_context.clone())
            {
                panic!(
                    "plugin event catalog namespace `{namespace}` should be globally unique; first declared by {previous_context}, repeated by {catalog_context} in {}",
                    relative_path.display()
                );
            }

            let version = integer_value(catalog, relative_path, &catalog_context, "version");
            assert!(
                version > 0 && version <= i64::from(u32::MAX),
                "plugin manifest {relative_path:?} {catalog_context} version `{version}` should be a positive u32"
            );
            assert_event_rows(catalog, relative_path, &catalog_context, namespace);
        }
    });
}
