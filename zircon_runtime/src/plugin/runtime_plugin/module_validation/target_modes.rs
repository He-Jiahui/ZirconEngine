mod presence;
mod row;
mod rows;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

use self::{
    presence::validate_runtime_plugin_module_target_mode_presence,
    rows::validate_runtime_plugin_module_target_mode_rows,
};

/// 校验模块声明的目标模式，而非替缺失声明推导默认目标。
/// 包调用端提供自身支持目标以约束模块；feature 调用端没有这一覆盖集合，传入 None。
/// 每个模块独立判重，多个模块面向同一目标是正常情况。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_target_modes(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_coverage: Option<(&str, &[RuntimeTargetMode])>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_target_mode_presence(manifest_label, module, diagnostics);
    validate_runtime_plugin_module_target_mode_rows(
        manifest_label,
        module,
        target_coverage,
        diagnostics,
    );
}
