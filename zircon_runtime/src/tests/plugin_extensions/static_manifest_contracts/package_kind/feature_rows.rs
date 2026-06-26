use super::super::for_each_static_plugin_manifest;
use super::package_kind_fields::{package_kind_value, table_array_row_count};

#[test]
fn plugin_tomls_declare_package_kind_feature_rows_coherently() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_kind = package_kind_value(table, relative_path);
        let optional_feature_count =
            table_array_row_count(table, relative_path, "optional_features");
        let feature_extension_count =
            table_array_row_count(table, relative_path, "feature_extensions");

        match package_kind {
            "standard" => assert_eq!(
                feature_extension_count, 0,
                "plugin manifest {relative_path:?} standard package_kind should not declare feature_extensions rows"
            ),
            "feature_extension" => {
                assert!(
                    feature_extension_count > 0,
                    "plugin manifest {relative_path:?} feature_extension package_kind should declare at least one feature_extensions row"
                );
                assert_eq!(
                    optional_feature_count, 0,
                    "plugin manifest {relative_path:?} feature_extension package_kind should not declare optional_features rows"
                );
            }
            _ => unreachable!("package_kind should already be validated"),
        }
    });
}
