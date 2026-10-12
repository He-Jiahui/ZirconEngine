use std::collections::BTreeMap;
use std::path::Path;

use super::super::{
    assert_non_empty_string, assert_non_empty_string_array, for_each_module_row,
    for_each_static_plugin_manifest, non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_module_identity() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            let module_kind =
                non_empty_string_value(module, relative_path, &module_context, "kind");
            assert!(
                matches!(module_kind, "runtime" | "editor" | "native" | "vm"),
                "plugin manifest {relative_path:?} {module_context} should declare a known module kind, got `{module_kind}`"
            );
            assert_non_empty_string(module, relative_path, &module_context, "crate_name");
        });
    });
}

fn assert_module_non_empty_string_array(
    module: &toml::Table,
    relative_path: &Path,
    module_context: &str,
    field_name: &str,
) {
    let module_name = module
        .get("name")
        .and_then(toml::Value::as_str)
        .unwrap_or("<unknown>");
    assert_non_empty_string_array(
        module,
        relative_path,
        &format!("{module_context} module `{module_name}`"),
        field_name,
    );
}

#[test]
fn plugin_tomls_declare_unique_module_names() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let mut module_names = BTreeMap::new();

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            if let Some(previous_context) =
                module_names.insert(module_name.to_string(), module_context.to_string())
            {
                panic!(
                    "plugin manifest {relative_path:?} module name `{module_name}` should be unique across package and optional-feature module rows; first declared in {previous_context}, repeated in {module_context}"
                );
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_module_capabilities() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            assert_module_non_empty_string_array(
                module,
                relative_path,
                module_context,
                "capabilities",
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_module_target_modes() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            assert_module_non_empty_string_array(
                module,
                relative_path,
                module_context,
                "target_modes",
            );
        });
    });
}
