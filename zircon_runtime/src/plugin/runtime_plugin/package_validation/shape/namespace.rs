mod segments;

use super::field::validate_runtime_plugin_package_field;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_namespace(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !validate_runtime_plugin_package_field(field_name, value, diagnostics) {
        return;
    }
    segments::validate_runtime_plugin_package_namespace_segments(field_name, value, diagnostics);
}
