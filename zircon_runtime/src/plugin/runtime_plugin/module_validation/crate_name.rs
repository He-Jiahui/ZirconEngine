mod shape;
mod token;
mod underscore;

use self::{
    shape::validate_runtime_plugin_module_crate_name_shape,
    token::validate_runtime_plugin_module_crate_name_token,
    underscore::validate_runtime_plugin_module_crate_name_underscore,
};

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_crate_name(
    manifest_label: &str,
    validate_field: fn(&str, &str, &mut Vec<String>),
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_crate_name_shape(validate_field, crate_name, diagnostics);
    validate_runtime_plugin_module_crate_name_token(manifest_label, crate_name, diagnostics);
    validate_runtime_plugin_module_crate_name_underscore(manifest_label, crate_name, diagnostics);
}
