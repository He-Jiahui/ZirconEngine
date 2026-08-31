use zircon_runtime::{
    core::framework::{platform::RuntimeTargetMode, project::ProjectPluginSelection},
    plugin::PluginModuleKind,
    plugin::PluginPackageManifest,
};

use super::module_crate_lookup::module_crate;

pub(in crate::ui::host::editor_manager_plugins_export) fn project_selection_from_package(
    package: &PluginPackageManifest,
) -> ProjectPluginSelection {
    let mut target_modes = Vec::new();
    let mut seen_target_modes = 0_u8;
    for target_mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        let bit = package_target_mode_bit(target_mode);
        if seen_target_modes & bit == 0 {
            seen_target_modes |= bit;
            target_modes.push(target_mode);
        }
    }
    ProjectPluginSelection {
        id: package.id.clone(),
        enabled: false,
        required: false,
        target_modes,
        packaging: zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: module_crate(package, PluginModuleKind::Runtime),
        editor_crate: module_crate(package, PluginModuleKind::Editor),
        features: Vec::new(),
    }
}

const fn package_target_mode_bit(target_mode: RuntimeTargetMode) -> u8 {
    match target_mode {
        RuntimeTargetMode::ClientRuntime => 0b001,
        RuntimeTargetMode::ServerRuntime => 0b010,
        RuntimeTargetMode::EditorHost => 0b100,
    }
}

#[cfg(test)]
#[path = "project_selection/bitset_tests.rs"]
mod bitset_tests;
