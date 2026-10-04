mod identity;
mod note;
mod references;
mod targets;

use super::super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::{CapabilityStatusManifest, PluginPackageManifest};

use self::{
    identity::validate_runtime_plugin_package_capability_status_row_identity,
    note::validate_runtime_plugin_package_capability_status_row_note,
    references::validate_runtime_plugin_package_capability_status_row_bevy_references,
    targets::validate_runtime_plugin_package_capability_status_row_targets,
};

/// 同一状态行的身份、目标范围、参考路径和备注分别积累诊断，避免一项无效掩盖其他问题。
pub(super) fn validate_runtime_plugin_package_capability_status_row(
    package_manifest: &PluginPackageManifest,
    status: &CapabilityStatusManifest,
    status_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_row_identity(
        status,
        status_index,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_row_targets(
        package_manifest,
        status,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_row_bevy_references(
        status,
        status_index,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_row_note(status, diagnostics);
}
