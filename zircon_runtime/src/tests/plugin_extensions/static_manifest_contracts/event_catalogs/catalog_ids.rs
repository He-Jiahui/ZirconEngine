use super::{
    assert_dot_namespaced_event_id, for_each_static_plugin_manifest, non_empty_string_value,
    visit_event_catalogs, visit_event_rows,
};

#[test]
fn plugin_tomls_declare_event_catalog_ids_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_event_catalogs(table, relative_path, &mut |catalog, catalog_context| {
            let namespace =
                non_empty_string_value(catalog, relative_path, "event catalog", "namespace");
            assert_dot_namespaced_event_id(relative_path, catalog_context, "namespace", namespace);

            visit_event_rows(
                catalog,
                relative_path,
                catalog_context,
                &mut |event, event_context| {
                    let event_id =
                        non_empty_string_value(event, relative_path, catalog_context, "id");
                    assert_dot_namespaced_event_id(relative_path, event_context, "id", event_id);
                },
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_event_catalog_namespaces_under_owner_package() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        visit_event_catalogs(table, relative_path, &mut |catalog, _| {
            let namespace =
                non_empty_string_value(catalog, relative_path, "event catalog", "namespace");
            let expected_prefix = format!("{package_id}.");
            assert!(
                namespace.starts_with(&expected_prefix),
                "plugin manifest {relative_path:?} event catalog namespace `{namespace}` should stay under package namespace `{expected_prefix}`"
            );
        });
    });
}
