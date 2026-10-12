pub(super) fn validate_runtime_plugin_package_semver_component_leading_zeroes(
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    let bytes = segment.as_bytes();
    if bytes.first() == Some(&b'0') && bytes.len() > 1 {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` {component_name} component `{segment}` must not use leading zeroes"
        ));
        return false;
    }
    true
}

#[cfg(test)]
#[path = "tests/leading_zeroes.rs"]
mod tests;
