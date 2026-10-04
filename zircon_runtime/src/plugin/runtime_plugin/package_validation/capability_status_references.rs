//! 能力状态中的 Bevy 参考是仓库源码证据路径；本层只检查声明形状与同一状态内的重复。
//! 校验不读取参考文件，也不确认路径存在，注册流程因而不依赖开发参考仓库已检出。

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
