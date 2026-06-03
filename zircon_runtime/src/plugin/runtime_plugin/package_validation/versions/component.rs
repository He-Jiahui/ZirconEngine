mod digits;
mod leading_zeroes;
mod range;

pub(super) fn validate_runtime_plugin_package_semver_component(
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
    diagnostics: &mut Vec<String>,
) {
    if !digits::validate_runtime_plugin_package_semver_component_digits(
        field_name,
        value,
        component_name,
        segment,
        diagnostics,
    ) {
        return;
    }
    if !leading_zeroes::validate_runtime_plugin_package_semver_component_leading_zeroes(
        field_name,
        value,
        component_name,
        segment,
        diagnostics,
    ) {
        return;
    }
    range::validate_runtime_plugin_package_semver_component_range(
        field_name,
        value,
        component_name,
        segment,
        diagnostics,
    );
}
