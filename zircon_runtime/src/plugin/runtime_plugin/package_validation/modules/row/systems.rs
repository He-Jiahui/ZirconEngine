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
    for value in values {
        validate_runtime_plugin_package_namespace(field_name, value, diagnostics);
        if !runtime_plugin_package_system_name_has_owner(package_id, value) {
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

fn runtime_plugin_package_system_name_has_owner(package_id: &str, value: &str) -> bool {
    value
        .strip_prefix(package_id)
        .is_some_and(|suffix| suffix.starts_with('.'))
}

#[cfg(test)]
mod tests {
    #[test]
    fn module_system_owner_check_does_not_format_a_prefix() {
        let source = include_str!("systems.rs");
        let formatted_prefix = ["format!(\"", "{package_id}.", "\")"].concat();
        assert!(!source.contains(&formatted_prefix));
    }

    #[test]
    fn module_system_owner_check_preserves_the_dot_boundary() {
        assert!(super::runtime_plugin_package_system_name_has_owner(
            "physics",
            "physics.simulation"
        ));
        assert!(!super::runtime_plugin_package_system_name_has_owner(
            "phys",
            "physics.simulation"
        ));
    }
}
