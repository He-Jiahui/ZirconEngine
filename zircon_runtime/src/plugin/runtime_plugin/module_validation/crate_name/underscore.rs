pub(super) fn validate_runtime_plugin_module_crate_name_underscore(
    manifest_label: &str,
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    if crate_name.ends_with('_') || crate_name.contains("__") {
        diagnostics.push(format!(
            "{manifest_label} module crate_name `{crate_name}` must not end with an underscore or contain repeated underscores"
        ));
    }
}
