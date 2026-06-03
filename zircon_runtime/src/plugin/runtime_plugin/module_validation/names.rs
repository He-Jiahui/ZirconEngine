mod kind_suffix;
mod owner_prefix;
mod shape;
mod uniqueness;

use crate::plugin::PluginModuleManifest;

use self::{
    kind_suffix::validate_runtime_plugin_module_name_kind_suffix,
    owner_prefix::validate_runtime_plugin_module_name_owner_prefix,
    shape::validate_runtime_plugin_module_name_shape,
    uniqueness::validate_runtime_plugin_module_name_uniqueness,
};

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_name<'a>(
    manifest_label: &str,
    owner_label: &str,
    owner_id: &str,
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    validate_field: fn(&str, &str, &mut Vec<String>),
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_name_shape(
        module,
        validate_field,
        validate_namespace,
        diagnostics,
    );
    validate_runtime_plugin_module_name_owner_prefix(
        manifest_label,
        owner_label,
        owner_id,
        module,
        diagnostics,
    );
    validate_runtime_plugin_module_name_kind_suffix(manifest_label, module, diagnostics);
    validate_runtime_plugin_module_name_uniqueness(manifest_label, module, seen_names, diagnostics);
}
