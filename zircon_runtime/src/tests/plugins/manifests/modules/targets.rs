use super::super::{
    for_each_module_row, for_each_static_plugin_manifest, non_empty_string_array_values,
    non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_module_targets_within_package_targets() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_targets =
            non_empty_string_array_values(table, relative_path, "top-level", "supported_targets");

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            for target_mode in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "target_modes",
            ) {
                assert!(
                    package_targets.contains(&target_mode),
                    "plugin manifest {relative_path:?} {module_context} target mode `{target_mode}` should be covered by package supported_targets"
                );
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_editor_modules_target_editor_host_only() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            let module_kind =
                non_empty_string_value(module, relative_path, &module_context, "kind");

            if module_kind != "editor" {
                return;
            }

            for target_mode in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "target_modes",
            ) {
                assert_eq!(
                    target_mode, "editor_host",
                    "plugin manifest {relative_path:?} {module_context} is an editor module and should only target editor_host, got `{target_mode}`"
                );
            }
        });
    });
}
