use crate::plugin::PluginModuleManifest;

pub(super) fn validate_runtime_plugin_module_name_shape(
    module: &PluginModuleManifest,
    validate_field: fn(&str, &str, &mut Vec<String>),
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    diagnostics: &mut Vec<String>,
) {
    validate_field("module name", &module.name, diagnostics);
    validate_namespace("module name", &module.name, diagnostics);
}
