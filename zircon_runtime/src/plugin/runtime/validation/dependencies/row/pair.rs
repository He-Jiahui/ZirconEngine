use super::super::pairs::validate_runtime_plugin_package_dependency_pair;

pub(super) fn validate_runtime_plugin_package_dependency_row_pair(
    dependency_id: &str,
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_dependency_pair(
        dependency_id,
        capability,
        is_duplicate,
        diagnostics,
    );
}
