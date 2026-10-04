mod count;

use self::count::validate_runtime_plugin_package_semver_segment_count;
use super::component::validate_runtime_plugin_package_semver_component;

// 先确认三段结构再逐段报告数值问题，避免对缺段或多段版本产生误导性的组件诊断。
pub(super) fn validate_runtime_plugin_package_semver_segments(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segments = value.split('.');
    let Some(major) = segments.next() else {
        validate_runtime_plugin_package_semver_segment_count(field_name, value, 0, diagnostics);
        return;
    };
    let Some(minor) = segments.next() else {
        validate_runtime_plugin_package_semver_segment_count(field_name, value, 1, diagnostics);
        return;
    };
    let Some(patch) = segments.next() else {
        validate_runtime_plugin_package_semver_segment_count(field_name, value, 2, diagnostics);
        return;
    };
    if segments.next().is_some() {
        validate_runtime_plugin_package_semver_segment_count(
            field_name,
            value,
            4 + segments.count(),
            diagnostics,
        );
        return;
    }
    for (component_name, segment) in ["major", "minor", "patch"]
        .into_iter()
        .zip([major, minor, patch])
    {
        validate_runtime_plugin_package_semver_component(
            field_name,
            value,
            component_name,
            segment,
            diagnostics,
        );
    }
}

#[cfg(test)]
#[path = "tests/segments.rs"]
mod tests;
