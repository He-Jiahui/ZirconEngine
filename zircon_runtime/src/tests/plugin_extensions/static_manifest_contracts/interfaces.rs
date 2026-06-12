use std::path::Path;

use super::{
    for_each_static_plugin_manifest, non_empty_string_array_values, non_empty_string_value,
    optional_table_array,
};

#[test]
fn plugin_tomls_declare_bridge_interface_namespaces() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for interface in
            optional_table_array(table, relative_path, "top-level", "provides_interfaces")
                .unwrap_or_default()
        {
            let interface_id =
                non_empty_string_value(interface, relative_path, "provided interface", "id");
            assert_interface_namespace(relative_path, "provided interface", interface_id);
        }

        for dependency in optional_table_array(table, relative_path, "top-level", "dependencies")
            .unwrap_or_default()
        {
            let dependency_id =
                non_empty_string_value(dependency, relative_path, "top-level dependency", "id");
            if !dependency.contains_key("interfaces") {
                continue;
            }
            for interface_id in non_empty_string_array_values(
                dependency,
                relative_path,
                &format!("top-level dependency `{dependency_id}`"),
                "interfaces",
            ) {
                assert_interface_namespace(
                    relative_path,
                    &format!("top-level dependency `{dependency_id}` interface"),
                    interface_id,
                );
            }
        }
    });
}

fn assert_interface_namespace(relative_path: &Path, context: &str, interface_id: &str) {
    assert_eq!(
        interface_id.trim(),
        interface_id,
        "plugin manifest {relative_path:?} {context} `{interface_id}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = interface_id.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} `{interface_id}` should use at least two dot-separated namespace segments"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} `{interface_id}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
            "plugin manifest {relative_path:?} {context} `{interface_id}` should use lowercase ASCII namespace segments"
        );
    }
}
