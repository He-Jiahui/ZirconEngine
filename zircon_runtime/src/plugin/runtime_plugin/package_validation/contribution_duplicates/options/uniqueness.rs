pub(super) fn validate_runtime_plugin_package_option_key_uniqueness<'a>(
    option_key: &'a str,
    seen_option_keys: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen_option_keys.contains(&option_key) {
        diagnostics.push(format!(
            "runtime plugin package manifest option key `{option_key}` must be unique"
        ));
    } else {
        seen_option_keys.push(option_key);
    }
}
