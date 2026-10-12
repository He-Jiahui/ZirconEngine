use super::super::{for_each_static_plugin_manifest, non_empty_string_value};

#[test]
fn plugin_tomls_declare_known_package_classification() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_package_category(table, relative_path);
        assert_known_plugin_maturity(table, relative_path);
    });
}

fn assert_known_package_category(table: &toml::Table, relative_path: &std::path::Path) {
    let category = non_empty_string_value(table, relative_path, "top-level", "category");
    assert!(
        matches!(
            category,
            "asset_importer"
                | "authoring"
                | "diagnostics"
                | "platform"
                | "rendering"
                | "runtime"
                | "sdk"
        ),
        "plugin manifest {relative_path:?} top-level category `{category}` should be a known package category"
    );
}

fn assert_known_plugin_maturity(table: &toml::Table, relative_path: &std::path::Path) {
    let maturity = non_empty_string_value(table, relative_path, "top-level", "maturity");
    assert!(
        matches!(
            maturity,
            "core" | "stable" | "beta" | "experimental" | "externalized" | "stub" | "deprecated"
        ),
        "plugin manifest {relative_path:?} top-level maturity `{maturity}` should be a known plugin maturity"
    );
}
