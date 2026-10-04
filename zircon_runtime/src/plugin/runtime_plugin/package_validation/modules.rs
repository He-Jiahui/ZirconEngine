//! 包模块行复用独立模块描述符的身份和目标规则，并用包投影核对包内命名与能力重复。
mod field;
mod row;
mod rows;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_modules(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_module_rows(package_manifest, projection, diagnostics);
}
