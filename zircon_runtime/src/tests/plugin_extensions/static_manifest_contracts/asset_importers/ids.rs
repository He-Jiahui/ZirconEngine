use std::path::Path;

use super::super::{for_each_static_plugin_manifest, non_empty_string_value};
use super::traversal::asset_importer_array;

pub(super) fn assert_dot_namespaced_importer_id(relative_path: &Path, importer_id: &str) {
    assert_eq!(
        importer_id.trim(),
        importer_id,
        "plugin manifest {relative_path:?} asset importer id `{importer_id}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = importer_id.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} asset importer id `{importer_id}` should use at least two dot-separated namespace segments"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} asset importer id `{importer_id}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} asset importer id `{importer_id}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}

#[test]
fn plugin_tomls_declare_asset_importer_ids_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(importers) = asset_importer_array(table, relative_path) else {
            return;
        };

        for importer in importers {
            let importer = importer.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} asset importer should be a table")
            });
            let importer_id =
                non_empty_string_value(importer, relative_path, "asset importer", "id");
            assert_dot_namespaced_importer_id(relative_path, importer_id);
        }
    });
}
