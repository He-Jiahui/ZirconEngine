use super::super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

use super::super::contribution_duplicates::{
    validate_duplicate_components, validate_duplicate_event_catalogs,
    validate_duplicate_plugin_options, validate_duplicate_ui_components,
};
use super::super::contribution_owners::{
    validate_component_owners, validate_event_catalog_owners, validate_ui_component_owners,
};

/// 汇总包贡献约束，先报告各域重复，再报告所有者错误；各阶段继续累计诊断。
/// 此顺序属于完整包校验的诊断契约，不能因发现一类错误而提前退出。
pub(super) fn validate_runtime_plugin_package_contribution_groups(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_duplicate_plugin_options(package_manifest, projection, diagnostics);
    validate_duplicate_event_catalogs(package_manifest, projection, diagnostics);
    validate_duplicate_components(package_manifest, projection, diagnostics);
    validate_duplicate_ui_components(package_manifest, projection, diagnostics);
    validate_event_catalog_owners(package_manifest, diagnostics);
    validate_component_owners(package_manifest, diagnostics);
    validate_ui_component_owners(package_manifest, diagnostics);
}
