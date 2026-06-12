use crate::plugin::PluginModuleManifest;

use super::super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_module_system_contracts(
    package_id: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_module_system_names(
        package_id,
        module,
        "system_set",
        &module.system_sets,
        diagnostics,
    );
    validate_runtime_plugin_package_module_system_names(
        package_id,
        module,
        "system_anchor",
        &module.system_anchors,
        diagnostics,
    );
}

fn validate_runtime_plugin_package_module_system_names(
    package_id: &str,
    module: &PluginModuleManifest,
    field_name: &str,
    values: &[String],
    diagnostics: &mut Vec<String>,
) {
    let mut seen = Vec::new();
    let expected_prefix = format!("{package_id}.");
    for value in values {
        validate_runtime_plugin_package_namespace(field_name, value, diagnostics);
        if !value.starts_with(&expected_prefix) {
            diagnostics.push(format!(
                "runtime plugin package manifest module `{}` {field_name} `{value}` must be prefixed by package id `{package_id}`",
                module.name
            ));
        }
        if seen.contains(&value.as_str()) {
            diagnostics.push(format!(
                "runtime plugin package manifest module `{}` {field_name} `{value}` must be unique",
                module.name
            ));
        } else {
            seen.push(value.as_str());
        }
    }
}
