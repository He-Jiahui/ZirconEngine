//! 每项能力状态的显式目标模式必须唯一，并落在所属包声明的支持范围内。
//! 空状态目标数组不会产生目标诊断；这里不会由包范围生成或补全状态目标。

mod coverage;
mod row;
mod rows;
mod uniqueness;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_capability_status_targets(
    package_manifest: &PluginPackageManifest,
    capability: &str,
    target_modes: &[RuntimeTargetMode],
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_capability_status_target_rows(
        package_manifest,
        capability,
        target_modes,
        diagnostics,
    );
}
