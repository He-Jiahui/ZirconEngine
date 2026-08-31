pub(super) fn validate_runtime_plugin_display_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() != value.len() {
        diagnostics.push(format!(
            "runtime plugin descriptor {field_name} `{value}` must be non-empty and trimmed"
        ));
    }
}

#[cfg(test)]
#[path = "display/single_trim_tests.rs"]
mod single_trim_tests;
