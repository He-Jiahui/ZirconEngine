use super::super::{
    assert_known_default_packaging_strategies, assert_non_empty_string_array,
    for_each_optional_feature, for_each_static_plugin_manifest,
};

#[test]
fn plugin_tomls_declare_default_packaging_options() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_non_empty_string_array(table, relative_path, "top-level", "default_packaging");
    });
}

#[test]
fn plugin_tomls_declare_known_default_packaging_strategies() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_default_packaging_strategies(table, relative_path, "top-level");

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            assert_known_default_packaging_strategies(feature, relative_path, feature_context);
        });
    });
}
