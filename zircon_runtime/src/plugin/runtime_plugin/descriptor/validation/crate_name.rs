use super::super::super::package_validation::is_lowercase_runtime_plugin_token;

pub(super) fn validate_runtime_plugin_descriptor_crate_name(
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    if crate_name.trim().is_empty()
        || crate_name.trim() != crate_name
        || !crate_name.starts_with("zircon_plugin_")
        || !is_lowercase_runtime_plugin_token(crate_name)
    {
        diagnostics.push(format!(
            "runtime plugin descriptor crate_name `{crate_name}` must use `zircon_plugin_` prefix and contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
    if crate_name.ends_with('_') || crate_name.contains("__") {
        diagnostics.push(format!(
            "runtime plugin descriptor crate_name `{crate_name}` must not end with an underscore or contain repeated underscores"
        ));
    }
}
