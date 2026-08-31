pub(in crate::plugin::runtime_plugin::feature_validation) fn validate_runtime_plugin_feature_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() != value.len() {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must be non-empty and trimmed"
        ));
    }
}

#[cfg(test)]
#[path = "field/single_trim_tests.rs"]
mod single_trim_tests;
