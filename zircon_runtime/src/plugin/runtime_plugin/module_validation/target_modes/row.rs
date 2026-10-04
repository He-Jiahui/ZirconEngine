mod coverage;
mod editor_host;
mod uniqueness;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

use self::{
    coverage::validate_runtime_plugin_module_target_mode_coverage,
    editor_host::validate_runtime_plugin_module_editor_host_target_mode,
    uniqueness::validate_runtime_plugin_module_target_mode_uniqueness,
};

/// 目标行的重复、宿主角色与包覆盖诊断彼此独立；发现重复仍需检查其可用范围。
/// seen 只属于当前模块的一次遍历，不能跨模块或跨次验证沿用。
pub(super) fn validate_runtime_plugin_module_target_mode_row(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_mode: RuntimeTargetMode,
    seen: &mut u8,
    target_coverage: Option<(&str, &[RuntimeTargetMode])>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_target_mode_uniqueness(
        manifest_label,
        module,
        target_mode,
        seen,
        diagnostics,
    );
    validate_runtime_plugin_module_editor_host_target_mode(
        manifest_label,
        module,
        target_mode,
        diagnostics,
    );
    validate_runtime_plugin_module_target_mode_coverage(
        manifest_label,
        module,
        target_mode,
        target_coverage,
        diagnostics,
    );
}
