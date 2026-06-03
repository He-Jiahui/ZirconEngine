use super::super::{
    assert_known_runtime_targets, for_each_module_row, for_each_static_plugin_manifest,
};

#[test]
fn plugin_tomls_declare_known_runtime_targets() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_runtime_targets(table, relative_path, "top-level", "supported_targets");

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = module
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            assert_known_runtime_targets(
                module,
                relative_path,
                &format!("{module_context} module `{module_name}`"),
                "target_modes",
            );
        });
    });
}
