use crate::plugin::PluginModuleManifest;

use super::super::super::super::super::module_validation::validate_runtime_plugin_module_name;
use super::super::super::super::validate_runtime_plugin_package_namespace;
use super::super::super::field::validate_runtime_plugin_package_module_field;

pub(super) fn validate_runtime_plugin_package_module_name<'a>(
    package_id: &str,
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_name(
        "runtime plugin package manifest",
        "package id",
        package_id,
        module,
        seen_names,
        validate_runtime_plugin_package_module_field,
        validate_runtime_plugin_package_namespace,
        diagnostics,
    );
}
