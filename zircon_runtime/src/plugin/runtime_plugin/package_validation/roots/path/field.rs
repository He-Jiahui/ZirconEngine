pub(super) fn validate_runtime_plugin_package_root_field(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    let trimmed = root.trim();
    if trimmed.is_empty() || trimmed.len() != root.len() {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be non-empty and trimmed"
        ));
        return false;
    }
    true
}

#[cfg(test)]
#[path = "field/single_trim_tests.rs"]
mod single_trim_tests;
