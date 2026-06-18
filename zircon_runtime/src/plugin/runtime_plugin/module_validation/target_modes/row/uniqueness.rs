use crate::builtin::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_target_mode_uniqueness(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_mode: RuntimeTargetMode,
    seen: &mut Vec<RuntimeTargetMode>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&target_mode) {
        diagnostics.push(format!(
            "{manifest_label} module `{}` target mode {target_mode:?} must be unique",
            module.name
        ));
    } else {
        seen.push(target_mode);
    }
}
