use zircon_runtime::{
    plugin::PluginModuleKind, plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
};

use super::module_crate_lookup::module_crate;

pub(in crate::ui::host::editor_manager_plugins_export) fn project_selection_from_package(
    package: &PluginPackageManifest,
) -> ProjectPluginSelection {
    let mut target_modes = Vec::new();
    for target_mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    ProjectPluginSelection {
        id: package.id.clone(),
        enabled: false,
        required: false,
        target_modes,
        packaging: zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: module_crate(package, PluginModuleKind::Runtime),
        editor_crate: module_crate(package, PluginModuleKind::Editor),
        features: Vec::new(),
    }
}
