use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_capability_presence(
    manifest_label: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    if module.capabilities.is_empty() {
        diagnostics.push(format!(
            "{manifest_label} module `{}` capabilities must declare at least one capability",
            module.name
        ));
    }
}
