use super::super::validate_runtime_plugin_package_field;

pub(super) fn validate_runtime_plugin_package_module_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let _ = validate_runtime_plugin_package_field(field_name, value, diagnostics);
}
