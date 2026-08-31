pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() != value.len() {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must be non-empty and trimmed"
        ));
        return false;
    }
    true
}

#[cfg(test)]
#[path = "field/single_trim_tests.rs"]
mod single_trim_tests;
