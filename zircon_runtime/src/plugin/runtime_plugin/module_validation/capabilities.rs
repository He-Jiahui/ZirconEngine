mod presence;
mod row;
mod rows;

use crate::plugin::PluginModuleManifest;

use self::{
    presence::validate_runtime_plugin_module_capability_presence,
    rows::validate_runtime_plugin_module_capability_rows,
};

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_capabilities(
    manifest_label: &str,
    module: &PluginModuleManifest,
    validate_field: Option<fn(&str, &str, &mut Vec<String>)>,
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    is_duplicate: impl Fn(usize) -> bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_capability_presence(manifest_label, module, diagnostics);
    validate_runtime_plugin_module_capability_rows(
        manifest_label,
        module,
        validate_field,
        validate_namespace,
        is_duplicate,
        diagnostics,
    );
}
