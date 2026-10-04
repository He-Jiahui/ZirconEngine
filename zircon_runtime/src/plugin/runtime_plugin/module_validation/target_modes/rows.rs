mod state;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

use self::state::new_runtime_plugin_module_target_mode_row_state;
use super::row::validate_runtime_plugin_module_target_mode_row;

/// 以模块为单位建立判重状态，保留声明顺序以稳定注册报告的诊断顺序。
/// 这里的重复声明诊断不同于项目选择投影中的目标并集，两条调用链不可互换。
pub(super) fn validate_runtime_plugin_module_target_mode_rows(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_coverage: Option<(&str, &[RuntimeTargetMode])>,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_module_target_mode_row_state();
    for target_mode in module.target_modes.iter().copied() {
        validate_runtime_plugin_module_target_mode_row(
            manifest_label,
            module,
            target_mode,
            &mut seen,
            target_coverage,
            diagnostics,
        );
    }
}
