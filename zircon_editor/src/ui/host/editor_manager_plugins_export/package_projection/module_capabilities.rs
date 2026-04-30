use zircon_runtime::{PluginModuleKind, PluginPackageManifest};

pub(in crate::ui::host::editor_manager_plugins_export) fn module_capabilities_for_package(
    package: &PluginPackageManifest,
    kind: PluginModuleKind,
) -> Vec<String> {
    package
        .modules
        .iter()
        .filter(|module| module.kind == kind)
        .flat_map(|module| module.capabilities.iter().cloned())
        .collect()
}

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
