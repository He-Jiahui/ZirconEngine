use std::collections::BTreeMap;
use std::path::Path;

use super::{for_each_static_plugin_manifest, integer_value, non_empty_string_value};

#[test]
fn plugin_tomls_declare_event_catalog_rows() {
    let mut namespaces = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(catalogs) = table.get("event_catalogs") else {
            return;
        };
        let catalogs = catalogs.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} event_catalogs should be an array")
        });
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

#[test]
fn plugin_tomls_declare_event_catalog_ids_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(catalogs) = table.get("event_catalogs") else {
            return;
        };
        let catalogs = catalogs.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} event_catalogs should be an array")
        });

        for catalog in catalogs {
            let catalog = catalog.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} event catalog should be a table")
            });
            let namespace =
                non_empty_string_value(catalog, relative_path, "event catalog", "namespace");
            let catalog_context = format!("event catalog `{namespace}`");
            assert_dot_namespaced_event_id(relative_path, &catalog_context, "namespace", namespace);

            let Some(events) = catalog.get("events") else {
                continue;
            };
            let events = events.as_array().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {catalog_context} events should be an array"
                )
            });
            for event in events {
                let event = event.as_table().unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {catalog_context} event should be a table"
                    )
                });
                let event_id = non_empty_string_value(event, relative_path, &catalog_context, "id");
                let event_context = format!("{catalog_context} event `{event_id}`");
                assert_dot_namespaced_event_id(relative_path, &event_context, "id", event_id);
            }
        }
    });
}

#[test]
fn plugin_tomls_declare_event_payload_schemas_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(catalogs) = table.get("event_catalogs") else {
            return;
        };
        let catalogs = catalogs.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} event_catalogs should be an array")
        });

        for catalog in catalogs {
            let catalog = catalog.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} event catalog should be a table")
            });
            let namespace =
                non_empty_string_value(catalog, relative_path, "event catalog", "namespace");
            let catalog_context = format!("event catalog `{namespace}`");

            let Some(events) = catalog.get("events") else {
                continue;
            };
            let events = events.as_array().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {catalog_context} events should be an array"
                )
            });
            for event in events {
                let event = event.as_table().unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {catalog_context} event should be a table"
                    )
                });
                let event_id = non_empty_string_value(event, relative_path, &catalog_context, "id");
                let event_context = format!("{catalog_context} event `{event_id}`");
                if let Some(_) = event.get("payload_schema") {
                    let payload_schema = non_empty_string_value(
                        event,
                        relative_path,
                        &event_context,
                        "payload_schema",
                    );
                    assert_dot_namespaced_event_id(
                        relative_path,
                        &event_context,
                        "payload_schema",
                        payload_schema,
                    );
                    assert_versioned_payload_schema(relative_path, &event_context, payload_schema);
                }
            }
        }
    });
}

#[test]
fn plugin_tomls_declare_event_payload_schemas_are_versioned() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(catalogs) = table.get("event_catalogs") else {
            return;
        };
        let catalogs = catalogs.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} event_catalogs should be an array")
        });

        for catalog in catalogs {
            let catalog = catalog.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} event catalog should be a table")
            });
            let namespace =
                non_empty_string_value(catalog, relative_path, "event catalog", "namespace");
            let catalog_context = format!("event catalog `{namespace}`");

            let Some(events) = catalog.get("events") else {
                continue;
            };
            let events = events.as_array().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {catalog_context} events should be an array"
                )
            });
            for event in events {
                let event = event.as_table().unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {catalog_context} event should be a table"
                    )
                });
                let event_id = non_empty_string_value(event, relative_path, &catalog_context, "id");
                let event_context = format!("{catalog_context} event `{event_id}`");
                if let Some(_) = event.get("payload_schema") {
                    let payload_schema = non_empty_string_value(
                        event,
                        relative_path,
                        &event_context,
                        "payload_schema",
                    );
                    assert_versioned_payload_schema(relative_path, &event_context, payload_schema);
                }
            }
        }
    });
}

fn assert_event_rows(
    catalog: &toml::Table,
    relative_path: &Path,
    catalog_context: &str,
    namespace: &str,
) {
    let events = catalog
        .get("events")
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {catalog_context} should declare `events` as an array"
            )
        });
    assert!(
        !events.is_empty(),
        "plugin manifest {relative_path:?} {catalog_context} events should not be empty"
    );

    let mut event_ids = BTreeMap::new();
    for event in events {
        let event = event.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} {catalog_context} event should be a table")
        });
        let event_id = non_empty_string_value(event, relative_path, catalog_context, "id");
        let event_context = format!("{catalog_context} event `{event_id}`");
        assert_dot_namespaced_event_id(relative_path, &event_context, "id", event_id);
        assert!(
            event_id.starts_with(&format!("{namespace}.")),
            "plugin manifest {relative_path:?} {event_context} id should stay under namespace `{namespace}`"
        );
        if let Some(previous_context) =
            event_ids.insert(event_id.to_string(), event_context.clone())
        {
            panic!(
                "plugin manifest {relative_path:?} event id `{event_id}` should be unique inside {catalog_context}; first declared by {previous_context}, repeated by {event_context}"
            );
        }

        let display_name =
            non_empty_string_value(event, relative_path, &event_context, "display_name");
        assert_trimmed(relative_path, &event_context, "display_name", display_name);
        if event.get("payload_schema").is_some() {
            let payload_schema =
                non_empty_string_value(event, relative_path, &event_context, "payload_schema");
            assert_dot_namespaced_event_id(
                relative_path,
                &event_context,
                "payload_schema",
                payload_schema,
            );
            assert_versioned_payload_schema(relative_path, &event_context, payload_schema);
        }
    }
}

fn assert_dot_namespaced_event_id(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
) {
    assert_trimmed(relative_path, context, field_name, value);

    let segments: Vec<_> = value.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should use at least two dot-separated namespace segments"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}

fn assert_versioned_payload_schema(relative_path: &Path, context: &str, payload_schema: &str) {
    let version_segment = payload_schema.rsplit('.').next().unwrap_or(payload_schema);
    let Some(version_number) = version_segment.strip_prefix('v') else {
        panic!(
            "plugin manifest {relative_path:?} {context} payload_schema `{payload_schema}` should end with a version segment like `v1`"
        );
    };
    assert!(
        !version_number.is_empty(),
        "plugin manifest {relative_path:?} {context} payload_schema `{payload_schema}` version segment should include digits"
    );
    assert!(
        version_number.bytes().all(|byte| byte.is_ascii_digit()),
        "plugin manifest {relative_path:?} {context} payload_schema `{payload_schema}` version segment should contain only digits after `v`"
    );
    assert!(
        !version_number.starts_with('0'),
        "plugin manifest {relative_path:?} {context} payload_schema `{payload_schema}` version segment should be a positive integer without leading zeroes"
    );
}

fn assert_trimmed(relative_path: &Path, context: &str, field_name: &str, value: &str) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {context} `{field_name}` should not have leading or trailing whitespace"
    );
}
