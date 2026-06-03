use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_name_owner_prefix(
    manifest_label: &str,
    owner_label: &str,
    owner_id: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    let module_prefix = format!("{owner_id}.");
    if !module.name.starts_with(&module_prefix) {
        diagnostics.push(format!(
            "{manifest_label} module name `{}` must be prefixed by {owner_label} `{owner_id}`",
            module.name
        ));
    }
}
