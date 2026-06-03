use super::super::super::super::super::module_validation::validate_runtime_plugin_module_crate_name;
use super::super::super::field::validate_runtime_plugin_package_module_field;

pub(super) fn validate_runtime_plugin_package_module_crate_name(
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_crate_name(
        "runtime plugin package manifest",
        validate_runtime_plugin_package_module_field,
        crate_name,
        diagnostics,
    );
}
