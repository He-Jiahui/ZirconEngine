use super::super::{
    assert_non_empty_string, assert_non_empty_string_array, for_each_static_plugin_manifest,
};

#[test]
fn plugin_tomls_declare_public_package_metadata() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in [
            "id",
            "version",
            "sdk_api_version",
            "display_name",
            "category",
            "description",
            "maturity",
        ] {
            assert_non_empty_string(table, relative_path, "top-level", field_name);
        }

        assert_non_empty_string_array(table, relative_path, "top-level", "supported_targets");
        assert_non_empty_string_array(table, relative_path, "top-level", "capabilities");
    });
}
