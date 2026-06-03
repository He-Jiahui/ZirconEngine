use std::collections::BTreeMap;

use super::super::{
    assert_unique_static_identity, for_each_optional_feature, for_each_static_plugin_manifest,
    non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_unique_package_ids() {
    let mut package_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        assert_unique_static_identity(
            &mut package_ids,
            package_id,
            format!("top-level package in {}", relative_path.display()),
        );
    });
}

#[test]
fn plugin_tomls_declare_unique_optional_feature_ids() {
    let mut static_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        assert_unique_static_identity(
            &mut static_ids,
            package_id,
            format!("top-level package in {}", relative_path.display()),
        );

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            assert_unique_static_identity(
                &mut static_ids,
                feature_id,
                format!("{feature_context} in {}", relative_path.display()),
            );
        });
    });
}
