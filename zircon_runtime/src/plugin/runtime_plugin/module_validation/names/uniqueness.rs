use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_name_uniqueness(
    manifest_label: &str,
    module: &PluginModuleManifest,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "{manifest_label} module name `{}` must be unique",
            module.name
        ));
    }
}
