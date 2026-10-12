pub(super) fn validate_runtime_plugin_package_option_key_uniqueness(
    option_key: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest option key `{option_key}` must be unique"
        ));
    }
}
