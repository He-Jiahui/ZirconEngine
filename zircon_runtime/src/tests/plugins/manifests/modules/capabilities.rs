use super::super::{
    for_each_module_row, for_each_static_plugin_manifest, non_empty_string_array_values,
    non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_module_capabilities_match_module_kind() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            let module_kind =
                non_empty_string_value(module, relative_path, &module_context, "kind");
            let expected_prefix = match module_kind {
                "runtime" => "runtime.",
                "editor" => "editor.",
                _ => return,
            };

            for capability in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "capabilities",
            ) {
                assert!(
                    capability.starts_with(expected_prefix),
                    "plugin manifest {relative_path:?} {module_context} kind `{module_kind}` capability `{capability}` should start with `{expected_prefix}`"
                );
            }
        });
    });
}
