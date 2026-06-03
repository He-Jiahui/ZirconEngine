use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_name_uniqueness<'a>(
    manifest_label: &str,
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    let module_name = module.name.as_str();
    if seen_names.contains(&module_name) {
        diagnostics.push(format!(
            "{manifest_label} module name `{}` must be unique",
            module.name
        ));
    } else {
        seen_names.push(module_name);
    }
}
