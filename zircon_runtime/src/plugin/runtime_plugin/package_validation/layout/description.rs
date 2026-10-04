use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_description(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if !package_manifest.description.is_empty()
        && has_outer_whitespace(&package_manifest.description)
    {
        diagnostics.push(format!(
            "runtime plugin package manifest description `{}` must be trimmed when present",
            package_manifest.description
        ));
    }
}

fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

#[cfg(test)]
#[path = "tests/description_optimization_tests.rs"]
mod optimization_tests;
