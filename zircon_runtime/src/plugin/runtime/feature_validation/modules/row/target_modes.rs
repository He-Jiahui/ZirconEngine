use crate::plugin::PluginModuleManifest;

use super::super::super::super::module_validation::validate_runtime_plugin_module_target_modes;

// 独立功能没有可供比对的包级目标集合；这里负责模块自身规则。
// 包内嵌功能的外层审查另行验证其模块目标被所属包覆盖。
pub(super) fn validate_runtime_plugin_feature_module_target_modes(
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_target_modes(
        "runtime plugin feature manifest",
        module,
        None,
        diagnostics,
    );
}
