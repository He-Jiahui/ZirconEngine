use super::super::super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::CapabilityStatusManifest;

use super::super::identity::validate_runtime_plugin_package_capability_status_identity;

pub(super) fn validate_runtime_plugin_package_capability_status_row_identity(
    status: &CapabilityStatusManifest,
    status_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_identity(
        status.capability.as_str(),
        projection.owns_capability(&status.capability),
        projection.capability_status_is_duplicate(status_index),
        diagnostics,
    );
}
