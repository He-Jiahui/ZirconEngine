mod count;

use self::count::validate_runtime_plugin_package_semver_segment_count;
use super::component::validate_runtime_plugin_package_semver_component;

pub(super) fn validate_runtime_plugin_package_semver_segments(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let segments = value.split('.').collect::<Vec<_>>();
    if !validate_runtime_plugin_package_semver_segment_count(
        field_name,
        value,
        segments.len(),
        diagnostics,
    ) {
        return;
    }
    for (component_name, segment) in ["major", "minor", "patch"].into_iter().zip(segments) {
        validate_runtime_plugin_package_semver_component(
            field_name,
            value,
            component_name,
            segment,
            diagnostics,
        );
    }
}
