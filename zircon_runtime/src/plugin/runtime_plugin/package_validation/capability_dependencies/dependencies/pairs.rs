pub(super) fn validate_runtime_plugin_package_dependency_pair<'a>(
    dependency_id: &'a str,
    capability: &'a str,
    seen: &mut Vec<(&'a str, &'a str)>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&(dependency_id, capability)) {
        diagnostics.push(format!(
            "runtime plugin package manifest dependency `{dependency_id}` capability `{capability}` must be unique",
        ));
    } else {
        seen.push((dependency_id, capability));
    }
}
