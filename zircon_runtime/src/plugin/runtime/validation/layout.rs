//! 这里汇总包的公开元数据、可选坐标、目标集合与相对根路径，再交给模块和特性校验处理其关联约束。
#[path = "layout/description.rs"]
mod description;
#[path = "layout/public_metadata.rs"]
mod public_metadata;
#[path = "layout/supported_platforms.rs"]
mod supported_platforms;
#[path = "layout/supported_targets.rs"]
mod supported_targets;

use crate::plugin::PluginPackageManifest;

use super::coordinates::validate_runtime_plugin_package_coordinates;
use super::projection::RuntimePluginPackageValidationProjection;
use super::roots::validate_runtime_plugin_package_roots;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_layout(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    public_metadata::validate_runtime_plugin_package_public_metadata(package_manifest, diagnostics);
    validate_runtime_plugin_package_coordinates(package_manifest, diagnostics);
    supported_targets::validate_runtime_plugin_package_supported_targets(
        package_manifest,
        diagnostics,
    );
    supported_platforms::validate_runtime_plugin_package_supported_platforms(
        package_manifest,
        diagnostics,
    );
    validate_runtime_plugin_package_roots(package_manifest, projection, diagnostics);
}
