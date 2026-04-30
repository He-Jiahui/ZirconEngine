use zircon_runtime::{PluginModuleKind, PluginPackageManifest};

pub(in crate::ui::host::editor_manager_plugins_export) fn module_crate(
    package: &PluginPackageManifest,
    kind: PluginModuleKind,
) -> Option<String> {
    package
        .modules
        .iter()
        .find(|module| module.kind == kind)
        .map(|module| module.crate_name.clone())
}
