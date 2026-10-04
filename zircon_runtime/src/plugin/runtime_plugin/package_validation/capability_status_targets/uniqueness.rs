use crate::core::framework::platform::RuntimeTargetMode;

/// 三种封闭目标模式以位集合表达同一状态内的重复，不改变调用方按声明顺序积累诊断的行为。
pub(super) fn validate_runtime_plugin_package_capability_status_target_uniqueness(
    capability: &str,
    target_mode: RuntimeTargetMode,
    seen: &mut u8,
    diagnostics: &mut Vec<String>,
) {
    let target_mode_bit = match target_mode {
        RuntimeTargetMode::ClientRuntime => 0b001,
        RuntimeTargetMode::ServerRuntime => 0b010,
        RuntimeTargetMode::EditorHost => 0b100,
    };
    if *seen & target_mode_bit != 0 {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` target mode {target_mode:?} must be unique"
        ));
    } else {
        *seen |= target_mode_bit;
    }
}
