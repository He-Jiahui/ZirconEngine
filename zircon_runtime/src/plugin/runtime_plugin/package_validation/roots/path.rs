mod field;
mod relative;
mod segments;
mod separator;

pub(super) fn validate_runtime_plugin_package_root(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    if !field::validate_runtime_plugin_package_root_field(field_name, root, diagnostics) {
        return;
    }
    relative::validate_runtime_plugin_package_root_relative(field_name, root, diagnostics);
    separator::validate_runtime_plugin_package_root_separator(field_name, root, diagnostics);
    segments::validate_runtime_plugin_package_root_segments(field_name, root, diagnostics);
}
