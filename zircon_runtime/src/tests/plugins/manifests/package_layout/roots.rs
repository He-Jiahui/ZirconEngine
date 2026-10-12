use std::path::Path;

use super::super::for_each_static_plugin_manifest;
use super::arrays::{assert_unique_entries, string_array_values};

#[test]
fn plugin_tomls_declare_package_root_arrays() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in ["asset_roots", "content_roots"] {
            let Some(value) = table.get(field_name) else {
                continue;
            };
            let roots = string_array_values(value, relative_path, "top-level", field_name);
            assert_unique_entries(relative_path, "top-level", field_name, &roots);

            for root in roots {
                assert_relative_package_root(relative_path, field_name, root);
            }
        }
    });
}

fn assert_relative_package_root(relative_path: &Path, field_name: &str, root: &str) {
    assert_eq!(
        root.trim(),
        root,
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should not have leading or trailing whitespace"
    );
    assert!(
        !root.starts_with('/') && !root.starts_with('\\'),
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should be relative"
    );
    assert!(
        !root.contains('\\'),
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should use forward slashes"
    );
    assert!(
        !root
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == ".."),
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should not contain empty, current, or parent path segments"
    );
}
