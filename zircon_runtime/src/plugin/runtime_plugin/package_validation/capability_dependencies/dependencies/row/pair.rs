use super::super::pairs::validate_runtime_plugin_package_dependency_pair;

pub(super) fn validate_runtime_plugin_package_dependency_row_pair<'a>(
    dependency_id: &'a str,
    capability: &'a str,
    seen: &mut Vec<(&'a str, &'a str)>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_dependency_pair(dependency_id, capability, seen, diagnostics);
}
