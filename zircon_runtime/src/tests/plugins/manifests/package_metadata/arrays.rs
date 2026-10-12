use super::super::{
    assert_unique_string_array_entries, for_each_module_row, for_each_optional_feature,
    for_each_static_plugin_manifest,
};

#[test]
fn plugin_tomls_declare_unique_string_array_entries() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in ["supported_targets", "capabilities", "default_packaging"] {
            assert_unique_string_array_entries(table, relative_path, "top-level", field_name);
        }

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            for field_name in ["capabilities", "default_packaging"] {
                assert_unique_string_array_entries(
                    feature,
                    relative_path,
                    feature_context,
                    field_name,
                );
            }
        });

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = module
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let module_context = format!("{module_context} module `{module_name}`");
            for field_name in ["capabilities", "target_modes"] {
                assert_unique_string_array_entries(
                    module,
                    relative_path,
                    &module_context,
                    field_name,
                );
            }
        });
    });
}
