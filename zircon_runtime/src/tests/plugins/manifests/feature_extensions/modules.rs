use std::collections::BTreeMap;

use super::{
    assert_crate_name_shape, assert_known_runtime_targets, assert_lowercase_dot_namespace,
    assert_unique_identity, assert_unique_string_array_entries, for_each_static_plugin_manifest,
    non_empty_string_array_values, non_empty_string_value, optional_table_array,
    visit_feature_extension_rows,
};

#[test]
fn plugin_tomls_declare_feature_extension_modules() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_targets =
            non_empty_string_array_values(table, relative_path, "top-level", "supported_targets");

        visit_feature_extension_rows(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            let feature_prefix = format!("{feature_id}.");
            let Some(modules) =
                optional_table_array(feature, relative_path, feature_context, "modules")
            else {
                return;
            };

            let mut module_names = BTreeMap::new();
            for module in modules {
                let module_name =
                    non_empty_string_value(module, relative_path, feature_context, "name");
                let module_context = format!("{feature_context} module `{module_name}`");
                assert_lowercase_dot_namespace(relative_path, &module_context, "name", module_name);
                assert!(
                    module_name.starts_with(&feature_prefix),
                    "plugin manifest {relative_path:?} {module_context} name `{module_name}` should stay under feature namespace `{feature_prefix}`"
                );
                assert_unique_identity(
                    relative_path,
                    &mut module_names,
                    module_name,
                    module_context.clone(),
                );

                let module_kind =
                    non_empty_string_value(module, relative_path, &module_context, "kind");
                let (module_suffix, capability_prefix) = match module_kind {
                    "runtime" => (".runtime", "runtime."),
                    "editor" => (".editor", "editor."),
                    _ => panic!(
                        "plugin manifest {relative_path:?} {module_context} kind `{module_kind}` should be runtime or editor"
                    ),
                };
                assert!(
                    module_name.ends_with(module_suffix),
                    "plugin manifest {relative_path:?} {module_context} kind `{module_kind}` should end with `{module_suffix}`"
                );

                let crate_name =
                    non_empty_string_value(module, relative_path, &module_context, "crate_name");
                assert_crate_name_shape(relative_path, &module_context, crate_name);

                for capability in non_empty_string_array_values(
                    module,
                    relative_path,
                    &module_context,
                    "capabilities",
                ) {
                    assert_lowercase_dot_namespace(
                        relative_path,
                        &module_context,
                        "capability",
                        capability,
                    );
                    assert!(
                        capability.starts_with(capability_prefix),
                        "plugin manifest {relative_path:?} {module_context} kind `{module_kind}` capability `{capability}` should start with `{capability_prefix}`"
                    );
                }
                assert_unique_string_array_entries(
                    module,
                    relative_path,
                    &module_context,
                    "capabilities",
                );

                assert_known_runtime_targets(
                    module,
                    relative_path,
                    &module_context,
                    "target_modes",
                );
                assert_unique_string_array_entries(
                    module,
                    relative_path,
                    &module_context,
                    "target_modes",
                );
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
                    if module_kind == "editor" {
                        assert_eq!(
                            target_mode, "editor_host",
                            "plugin manifest {relative_path:?} {module_context} is an editor module and should only target editor_host, got `{target_mode}`"
                        );
                    }
                }
            }
        });
    });
}
