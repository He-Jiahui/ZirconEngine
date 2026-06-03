pub(super) fn validate_runtime_plugin_module_crate_name_shape(
    validate_field: fn(&str, &str, &mut Vec<String>),
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_field("module crate_name", crate_name, diagnostics);
}
