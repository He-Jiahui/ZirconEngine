pub(super) fn validate_runtime_plugin_package_root_relative(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    let has_windows_unsafe_component = root
        .replace('\\', "/")
        .split('/')
        .any(is_windows_unsafe_component);
    if root
        .as_bytes()
        .first()
        .is_some_and(|byte| matches!(byte, b'/' | b'\\'))
        || has_windows_unsafe_component
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be a portable relative path"
        ));
    }
}

fn is_windows_unsafe_component(component: &str) -> bool {
    if component.contains(':') {
        return true;
    }
    let normalized = component.trim_end_matches([' ', '.']);
    let stem = normalized
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|prefix| {
        stem.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}

#[cfg(test)]
#[path = "tests/relative.rs"]
mod tests;
