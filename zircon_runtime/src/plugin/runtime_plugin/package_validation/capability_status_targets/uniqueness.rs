use crate::builtin::RuntimeTargetMode;

pub(super) fn validate_runtime_plugin_package_capability_status_target_uniqueness(
    capability: &str,
    target_mode: RuntimeTargetMode,
    seen: &mut Vec<RuntimeTargetMode>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&target_mode) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` target mode {target_mode:?} must be unique"
        ));
    } else {
        seen.push(target_mode);
    }
}
