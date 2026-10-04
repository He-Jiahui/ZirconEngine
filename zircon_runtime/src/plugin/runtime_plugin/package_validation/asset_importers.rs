//! 包清单中的导入器先形成注册报告诊断，再由注册阶段纳入扩展表；这里检查声明身份，不替代导入器注册表对来源扩展名的校验。
mod identity;
mod required_capabilities;
mod row;
mod rows;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_asset_importers(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_asset_importer_rows(
        package_manifest,
        projection,
        diagnostics,
    );
}
