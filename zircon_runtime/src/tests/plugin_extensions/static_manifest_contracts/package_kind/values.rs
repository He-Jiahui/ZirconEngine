use super::super::for_each_static_plugin_manifest;
use super::helpers::package_kind_value;

#[test]
fn plugin_tomls_declare_known_package_kind_values() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_kind = package_kind_value(table, relative_path);
        assert!(
            matches!(package_kind, "standard" | "feature_extension"),
            "plugin manifest {relative_path:?} package_kind `{package_kind}` should be standard or feature_extension"
        );
    });
}
