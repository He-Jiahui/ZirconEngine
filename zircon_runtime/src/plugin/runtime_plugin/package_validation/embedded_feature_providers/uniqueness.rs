pub(super) fn validate_runtime_plugin_package_feature_provider_uniqueness(
    field_name: &str,
    feature_id: &str,
    provider_package_id: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{feature_id}` provider `{provider_package_id}` must be unique",
        ));
    }
}

#[cfg(test)]
#[path = "tests/uniqueness.rs"]
mod tests;
