use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ProjectPluginFeatureSelection,
    core::framework::project::ProjectPluginSelection, plugin::PluginFeatureBundleManifest,
    plugin::PluginModuleKind, plugin::PluginPackageManifest,
};

use super::module_crate_lookup::module_crate;

pub(in crate::ui::host::editor_manager_plugins_export) fn native_project_selection(
    package: &PluginPackageManifest,
) -> ProjectPluginSelection {
    let mut target_modes = Vec::new();
    let mut seen_target_modes = 0_u8;
    for target_mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        let bit = native_target_mode_bit(target_mode);
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
        packaging: zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
        runtime_crate: module_crate(package, PluginModuleKind::Runtime),
        editor_crate: module_crate(package, PluginModuleKind::Editor),
        features: package
            .optional_features
            .iter()
            .map(native_project_feature_selection)
            .collect(),
    }
}

fn native_project_feature_selection(
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut selection = ProjectPluginFeatureSelection::new(feature.id.clone())
        .enabled(feature.enabled_by_default)
        .with_packaging(default_feature_packaging(feature))
        .with_target_modes(feature_target_modes(feature));
    if let Some(crate_name) = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .map(|module| module.crate_name.clone())
    {
        selection = selection.with_runtime_crate(crate_name);
    }
    if let Some(crate_name) = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Editor)
        .map(|module| module.crate_name.clone())
    {
        selection = selection.with_editor_crate(crate_name);
    }
    selection
}

fn default_feature_packaging(feature: &PluginFeatureBundleManifest) -> ExportPackagingStrategy {
    feature
        .default_packaging
        .iter()
        .copied()
        .find(|packaging| *packaging == ExportPackagingStrategy::NativeDynamic)
        .or_else(|| feature.default_packaging.first().copied())
        .unwrap_or(ExportPackagingStrategy::NativeDynamic)
}

fn feature_target_modes(feature: &PluginFeatureBundleManifest) -> Vec<RuntimeTargetMode> {
    let mut target_modes = Vec::new();
    let mut seen_target_modes = 0_u8;
    for target_mode in feature
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        let bit = native_target_mode_bit(target_mode);
        if seen_target_modes & bit == 0 {
            seen_target_modes |= bit;
            target_modes.push(target_mode);
        }
    }
    target_modes
}

const fn native_target_mode_bit(target_mode: RuntimeTargetMode) -> u8 {
    match target_mode {
        RuntimeTargetMode::ClientRuntime => 0b001,
        RuntimeTargetMode::ServerRuntime => 0b010,
        RuntimeTargetMode::EditorHost => 0b100,
    }
}

#[cfg(test)]
#[path = "native_project_selection/tests/bitset_tests.rs"]
mod bitset_tests;

#[cfg(test)]
#[path = "tests/native_project_selection.rs"]
mod tests;
