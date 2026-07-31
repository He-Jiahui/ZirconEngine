use super::super::super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginModuleManifest;

use super::super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_module_system_contracts(
    package_id: &str,
    module: &PluginModuleManifest,
    module_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_module_system_names(
        package_id,
        module,
        "system_set",
        &module.system_sets,
        |index| projection.package_module_system_set_is_duplicate(module_index, index),
        diagnostics,
    );
    validate_runtime_plugin_package_module_system_names(
        package_id,
        module,
        "system_anchor",
        &module.system_anchors,
        |index| projection.package_module_system_anchor_is_duplicate(module_index, index),
        diagnostics,
    );
}

fn validate_runtime_plugin_package_module_system_names(
    package_id: &str,
    module: &PluginModuleManifest,
    field_name: &str,
    values: &[String],
    is_duplicate: impl Fn(usize) -> bool,
    diagnostics: &mut Vec<String>,
) {
    for (index, value) in values.iter().enumerate() {
        validate_runtime_plugin_package_namespace(field_name, value, diagnostics);
        if !runtime_plugin_package_system_name_has_owner(package_id, value) {
            diagnostics.push(format!(
                "runtime plugin package manifest module `{}` {field_name} `{value}` must be prefixed by package id `{package_id}`",
                module.name
            ));
        }
        if is_duplicate(index) {
            diagnostics.push(format!(
                "runtime plugin package manifest module `{}` {field_name} `{value}` must be unique",
                module.name
            ));
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
