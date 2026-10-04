use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginPackageManifest;

// TODO: [CR-PLUGIN-VALIDATION-0301] 确认空包支持目标是否允许显式状态目标；本运行时检查会拒绝，导出校验在包支持目标为空时跳过覆盖检查，需补同一清单的两入口对照用例。
pub(super) fn validate_runtime_plugin_package_capability_status_target_coverage(
    package_manifest: &PluginPackageManifest,
    capability: &str,
    target_mode: RuntimeTargetMode,
    diagnostics: &mut Vec<String>,
) {
    if !package_manifest.supported_targets.contains(&target_mode) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` target mode {target_mode:?} must be covered by package supported_targets"
        ));
    }
}
