mod field;
mod path;
mod row;
mod rows;
mod uniqueness;

use super::projection::RuntimePluginPackageValidationProjection;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_capability_status_bevy_references(
    capability: &str,
    bevy_references: &[String],
    status_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_capability_status_bevy_reference_rows(
        capability,
        bevy_references,
        status_index,
        projection,
        diagnostics,
    );
}
