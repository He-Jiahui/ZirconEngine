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
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    }
}
