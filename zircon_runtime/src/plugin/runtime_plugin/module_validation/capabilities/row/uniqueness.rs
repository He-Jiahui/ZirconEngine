use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_capability_uniqueness(
    manifest_label: &str,
    module: &PluginModuleManifest,
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "{manifest_label} module `{}` capability `{capability}` must be unique",
            module.name
        ));
    }
}
