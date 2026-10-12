use super::{
    assert_dot_namespaced_event_id, assert_versioned_payload_schema,
    for_each_static_plugin_manifest, non_empty_string_value, visit_event_catalogs,
    visit_event_rows,
};

#[test]
fn plugin_tomls_declare_event_payload_schemas_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_event_catalogs(table, relative_path, &mut |catalog, catalog_context| {
            visit_event_rows(
                catalog,
                relative_path,
                catalog_context,
                &mut |event, event_context| {
                    if event.get("payload_schema").is_some() {
                        let payload_schema = non_empty_string_value(
                            event,
                            relative_path,
                            event_context,
                            "payload_schema",
                        );
                        assert_dot_namespaced_event_id(
                            relative_path,
                            event_context,
                            "payload_schema",
                            payload_schema,
                        );
                        assert_versioned_payload_schema(
                            relative_path,
                            event_context,
                            payload_schema,
                        );
                    }
                },
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_event_payload_schemas_under_owner_package() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        visit_event_catalogs(table, relative_path, &mut |catalog, catalog_context| {
            visit_event_rows(
                catalog,
                relative_path,
                catalog_context,
                &mut |event, event_context| {
                    if event.get("payload_schema").is_some() {
                        let payload_schema = non_empty_string_value(
                            event,
                            relative_path,
                            event_context,
                            "payload_schema",
                        );
                        let expected_prefix = format!("{package_id}.");
                        assert!(
                            payload_schema.starts_with(&expected_prefix),
                            "plugin manifest {relative_path:?} {event_context} payload_schema `{payload_schema}` should stay under package namespace `{expected_prefix}`"
                        );
                    }
                },
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_event_payload_schemas_are_versioned() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_event_catalogs(table, relative_path, &mut |catalog, catalog_context| {
            visit_event_rows(
                catalog,
                relative_path,
                catalog_context,
                &mut |event, event_context| {
                    if event.get("payload_schema").is_some() {
                        let payload_schema = non_empty_string_value(
                            event,
                            relative_path,
                            event_context,
                            "payload_schema",
                        );
                        assert_versioned_payload_schema(
                            relative_path,
                            event_context,
                            payload_schema,
                        );
                    }
                },
            );
        });
    });
}
