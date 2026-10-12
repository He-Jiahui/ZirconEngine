use super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_capability_status_bevy_reference_row;

/// 路径重复只在当前状态行内判定；不同能力可以共享同一份参考源码证据。
pub(super) fn validate_runtime_plugin_package_capability_status_bevy_reference_rows(
    capability: &str,
    bevy_references: &[String],
    status_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (reference_index, reference) in bevy_references.iter().enumerate() {
        validate_runtime_plugin_package_capability_status_bevy_reference_row(
            capability,
            reference,
            projection.capability_status_reference_is_duplicate(status_index, reference_index),
            diagnostics,
        );
    }
}
