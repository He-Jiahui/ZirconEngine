use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_target_mode_uniqueness(
    manifest_label: &str,
    module: &PluginModuleManifest,
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
            "{manifest_label} module `{}` target mode {target_mode:?} must be unique",
            module.name
        ));
    } else {
        *seen |= target_mode_bit;
    }
}
