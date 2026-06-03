use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_capability_uniqueness<'a>(
    manifest_label: &str,
    module: &PluginModuleManifest,
    capability: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&capability) {
        diagnostics.push(format!(
            "{manifest_label} module `{}` capability `{capability}` must be unique",
            module.name
        ));
    } else {
        seen.push(capability);
    }
}
