use crate::plugin::PluginModuleManifest;

use super::super::super::super::super::module_validation::validate_runtime_plugin_module_name;
use super::super::super::super::validate_runtime_plugin_package_namespace;
use super::super::super::field::validate_runtime_plugin_package_module_field;

pub(super) fn validate_runtime_plugin_package_module_name(
    package_id: &str,
    module: &PluginModuleManifest,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_name(
        "runtime plugin package manifest",
        "package id",
        package_id,
        module,
        is_duplicate,
        validate_runtime_plugin_package_module_field,
        validate_runtime_plugin_package_namespace,
        diagnostics,
    );
}
