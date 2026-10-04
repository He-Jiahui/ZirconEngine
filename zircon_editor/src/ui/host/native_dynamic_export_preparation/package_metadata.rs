use zircon_runtime::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

pub(super) fn module_crate(
    package: &PluginPackageManifest,
    kind: PluginModuleKind,
) -> Option<String> {
    package
        .modules
        .iter()
        .find(|module| module.kind == kind)
        .map(|module| module.crate_name.clone())
}

pub(super) fn sanitize_path_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        sanitized.push(if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            ch
        } else {
            '_'
        });
    }
    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
#[path = "tests/package_metadata_performance_tests.rs"]
mod performance_tests;
