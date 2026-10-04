use zircon_runtime::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

pub(in crate::ui::host::editor_manager_plugins_export) fn module_capabilities_for_package(
    package: &PluginPackageManifest,
    kind: PluginModuleKind,
) -> Vec<String> {
    let capacity = package
        .modules
        .iter()
        .filter(|module| module.kind == kind)
        .map(|module| module.capabilities.len())
        .sum();
    let mut capabilities = Vec::with_capacity(capacity);
    capabilities.extend(
        package
            .modules
            .iter()
            .filter(|module| module.kind == kind)
            .flat_map(|module| module.capabilities.iter().cloned()),
    );
    capabilities
}

#[cfg(test)]
#[path = "tests/module_capabilities.rs"]
mod tests;

pub(in crate::ui::host::editor_manager_plugins_export) fn runtime_capabilities_for_package(
    package: &PluginPackageManifest,
) -> Vec<String> {
    module_capabilities_for_package(package, PluginModuleKind::Runtime)
}

pub(in crate::ui::host::editor_manager_plugins_export) fn editor_capabilities_for_package(
    package: &PluginPackageManifest,
) -> Vec<String> {
    module_capabilities_for_package(package, PluginModuleKind::Editor)
}
