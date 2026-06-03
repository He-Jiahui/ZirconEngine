pub(super) fn validate_runtime_plugin_package_bevy_reference_uniqueness<'a>(
    capability: &str,
    reference: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&reference) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must be unique"
        ));
    } else {
        seen.push(reference);
    }
}
