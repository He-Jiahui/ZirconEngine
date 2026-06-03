use std::collections::BTreeMap;

use super::super::{
    assert_unique_static_identity, assert_unique_string_array_entries,
    for_each_static_plugin_manifest, integer_value, non_empty_string_array_values,
    non_empty_string_value,
};
use super::kinds::{assert_known_asset_kind, assert_known_asset_kind_value};
use super::selectors::assert_asset_importer_source_selectors;
use super::traversal::asset_importer_array;

#[test]
fn plugin_tomls_declare_asset_importer_rows() {
    let mut importer_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(importers) = asset_importer_array(table, relative_path) else {
            return;
        };
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        assert!(
            !importers.is_empty(),
            "plugin manifest {relative_path:?} asset_importers should not be empty when declared"
        );

        for importer in importers {
            let importer = importer.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} asset importer should be a table")
            });
            let importer_id =
                non_empty_string_value(importer, relative_path, "asset importer", "id");
            let importer_context = format!("asset importer `{importer_id}`");
            assert_unique_static_identity(
                &mut importer_ids,
                importer_id,
                format!("{importer_context} in {}", relative_path.display()),
            );

            let plugin_id =
                non_empty_string_value(importer, relative_path, &importer_context, "plugin_id");
            assert_eq!(
                plugin_id, package_id,
                "plugin manifest {relative_path:?} {importer_context} plugin_id should match package id `{package_id}`"
            );

            let priority = integer_value(importer, relative_path, &importer_context, "priority");
            assert!(
                priority >= i64::from(i32::MIN) && priority <= i64::from(i32::MAX),
                "plugin manifest {relative_path:?} {importer_context} priority `{priority}` should fit i32"
            );
            let importer_version = integer_value(
                importer,
                relative_path,
                &importer_context,
                "importer_version",
            );
            assert!(
                importer_version > 0 && importer_version <= i64::from(u32::MAX),
                "plugin manifest {relative_path:?} {importer_context} importer_version `{importer_version}` should be a positive u32"
            );

            assert_known_asset_kind(importer, relative_path, &importer_context, "output_kind");
            assert_asset_importer_source_selectors(importer, relative_path, &importer_context);
            assert_unique_string_array_entries(
                importer,
                relative_path,
                &importer_context,
                "required_capabilities",
            );

            if importer.get("additional_output_kinds").is_some() {
                assert_unique_string_array_entries(
                    importer,
                    relative_path,
                    &importer_context,
                    "additional_output_kinds",
                );
                for output_kind in non_empty_string_array_values(
                    importer,
                    relative_path,
                    &importer_context,
                    "additional_output_kinds",
                ) {
                    assert_known_asset_kind_value(
                        output_kind,
                        relative_path,
                        &importer_context,
                        "additional_output_kinds",
                    );
                }
            }
        }
    });
}
