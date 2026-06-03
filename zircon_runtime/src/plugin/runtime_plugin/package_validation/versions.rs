mod component;
mod field;
mod segments;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_semver(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !field::validate_runtime_plugin_package_semver_field(field_name, value, diagnostics) {
        return;
    }
    segments::validate_runtime_plugin_package_semver_segments(field_name, value, diagnostics);
}
