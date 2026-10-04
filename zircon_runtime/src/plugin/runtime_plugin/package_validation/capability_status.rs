//! 包级能力状态的元数据校验，供注册报告说明能力的实现状态与参考依据。
//! 状态只能引用同包声明的包能力或可选特性能力；实际特性注册由目录与注册报告另行核对。

mod identity;
mod note;
mod row;
mod rows;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

// TODO: [CR-PLUGIN-VALIDATION-0300] 确认运行时是否应与导出校验一致拒绝显式空状态数组及空可选数组；当前 Vec 反序列化丢失缺省与显式空数组的区别，需对照导出空数组契约补跨入口用例。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_capability_statuses(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_capability_status_rows(
        package_manifest,
        projection,
        diagnostics,
    );
}
