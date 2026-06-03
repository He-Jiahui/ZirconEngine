use super::super::{
    assert_non_empty_string, assert_non_empty_string_array, bool_value, for_each_optional_feature,
    for_each_static_plugin_manifest, non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_optional_feature_metadata() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            assert_eq!(
                feature
                    .get("owner_plugin_id")
                    .and_then(toml::Value::as_str),
                Some(package_id),
                "plugin manifest {relative_path:?} {feature_context} should declare owner_plugin_id matching package id `{package_id}`"
            );
            for field_name in ["id", "display_name"] {
                assert_non_empty_string(feature, relative_path, feature_context, field_name);
            }
            for field_name in ["capabilities", "default_packaging"] {
                assert_non_empty_string_array(feature, relative_path, feature_context, field_name);
            }
            bool_value(
                feature,
                relative_path,
                feature_context,
                "enabled_by_default",
            );
        });
    });
}
