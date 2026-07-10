use crate::builtin::RuntimeTargetMode;
use crate::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginModuleKind, ProjectPluginManifest,
    ProjectPluginSelection,
};

use super::super::feature_definition_collection::feature_definition_map;
use super::super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

pub(super) fn complete_external_feature_provider_selections(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    completed: &mut ProjectPluginManifest,
) {
    let definitions = feature_definition_map(registrations, feature_registrations);
    for key in &definitions.definition_order {
        let Some(definition) = definitions.definitions.get(key) else {
            continue;
        };
        if definition.provider_package_id == definition.manifest.owner_plugin_id {
            continue;
        }
        if completed
            .selections
            .iter()
            .any(|selection| selection.id == definition.provider_package_id)
        {
            continue;
        }
        completed.selections.push(provider_selection(
            &definition.provider_package_id,
            &definition.manifest,
        ));
    }
}

fn provider_selection(
    provider_package_id: &str,
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: provider_package_id.to_string(),
        enabled: false,
        required: false,
        target_modes: feature_target_modes(feature),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: feature
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Runtime)
            .map(|module| module.crate_name.clone()),
        editor_crate: feature
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .map(|module| module.crate_name.clone()),
        features: Vec::new(),
    }
}

fn feature_target_modes(feature: &PluginFeatureBundleManifest) -> Vec<RuntimeTargetMode> {
    let mut target_modes = Vec::new();
    for target in feature
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target) {
            target_modes.push(target);
        }
    }
    target_modes
}
