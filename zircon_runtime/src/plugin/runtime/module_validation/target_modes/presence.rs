use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_target_mode_presence(
    manifest_label: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    if module.target_modes.is_empty() {
        diagnostics.push(format!(
            "{manifest_label} module `{}` target_modes must declare at least one target mode",
            module.name
        ));
    }
}
