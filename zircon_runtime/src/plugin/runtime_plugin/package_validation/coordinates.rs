//! 可选的包坐标三元组用于生成发布坐标；校验保留未声明坐标时沿用原包 ID 的调用契约。
mod fields;
mod presence;
mod shape;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_coordinates(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    fields::validate_runtime_plugin_package_coordinate_fields(package_manifest, diagnostics);
}
