use crate::core::framework::platform::RuntimeTargetMode;

pub(super) fn validate_runtime_plugin_package_supported_target_uniqueness(
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
            "runtime plugin package manifest supported_targets target mode {target_mode:?} must be unique"
        ));
    } else {
        *seen |= target_mode_bit;
    }
}
