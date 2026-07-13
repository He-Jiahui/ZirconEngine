use crate::core::framework::platform::RuntimeTargetMode;

pub(super) fn validate_runtime_plugin_package_supported_target_uniqueness(
    target_mode: RuntimeTargetMode,
    seen: &mut Vec<RuntimeTargetMode>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&target_mode) {
        diagnostics.push(format!(
            "runtime plugin package manifest supported_targets target mode {target_mode:?} must be unique"
        ));
    } else {
        seen.push(target_mode);
    }
}
