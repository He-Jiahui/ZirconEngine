mod kind_prefix;
mod uniqueness;

use crate::plugin::PluginModuleManifest;

use self::{
    kind_prefix::validate_runtime_plugin_module_capability_kind_prefix,
    uniqueness::validate_runtime_plugin_module_capability_uniqueness,
};

pub(super) fn validate_runtime_plugin_module_capability_row<'a>(
    manifest_label: &str,
    module: &PluginModuleManifest,
    capability: &'a str,
    seen: &mut Vec<&'a str>,
    validate_field: Option<fn(&str, &str, &mut Vec<String>)>,
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    diagnostics: &mut Vec<String>,
) {
    if let Some(validate_field) = validate_field {
        validate_field("module capability", capability, diagnostics);
    }
    validate_namespace("module capability", capability, diagnostics);
    validate_runtime_plugin_module_capability_kind_prefix(
        manifest_label,
        module,
        capability,
        diagnostics,
    );
    validate_runtime_plugin_module_capability_uniqueness(
        manifest_label,
        module,
        capability,
        seen,
        diagnostics,
    );
}
