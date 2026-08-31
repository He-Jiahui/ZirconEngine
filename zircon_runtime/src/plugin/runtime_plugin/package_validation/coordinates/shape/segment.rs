use super::super::super::is_lowercase_runtime_plugin_token;

pub(in crate::plugin::runtime_plugin::package_validation::coordinates) fn validate_runtime_plugin_package_coordinate_segment(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.len() != value.len()
        || !is_lowercase_runtime_plugin_token(value)
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must be a non-empty lowercase coordinate segment"
        ));
    }
}

#[cfg(test)]
#[path = "segment/single_trim_tests.rs"]
mod single_trim_tests;
