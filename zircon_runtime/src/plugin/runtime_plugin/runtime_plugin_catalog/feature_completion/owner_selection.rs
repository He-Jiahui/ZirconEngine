use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
use crate::plugin::PluginFeatureBundleManifest;

use super::super::super::feature_registration_report::project_selection_from_feature_manifest;
use super::super::feature_definitions::FeatureDefinition;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete_owner_feature_selection(
    owner_selection: &mut ProjectPluginSelection,
    feature: &PluginFeatureBundleManifest,
    provider_package_id: Option<&str>,
) {
    let mut catalog_selection = project_selection_from_feature_manifest(feature);
    if let Some(provider_package_id) = provider_package_id {
        catalog_selection.provider_package_id = Some(provider_package_id.to_string());
    }
    if let Some(selection) = owner_selection
        .features
        .iter_mut()
        .find(|selection| selection.id == catalog_selection.id)
    {
        if selection.runtime_crate.is_none() {
            selection.runtime_crate = catalog_selection.runtime_crate;
        }
        if selection.editor_crate.is_none() {
            selection.editor_crate = catalog_selection.editor_crate;
        }
        if selection.target_modes.is_empty() {
            selection.target_modes = catalog_selection.target_modes;
        }
        if selection.provider_package_id.is_none() {
            selection.provider_package_id = catalog_selection.provider_package_id;
        }
        return;
    }
    owner_selection.features.push(catalog_selection);
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete_external_provider_selection(
    manifest: &mut ProjectPluginManifest,
    definition: &FeatureDefinition,
) {
    let Some(provider_package_id) = definition.external_provider_for_owner() else {
        return;
    };
    if manifest
        .selections
        .iter()
        .any(|selection| selection.id == provider_package_id)
    {
        return;
    }
    let feature_selection = project_selection_from_feature_manifest(&definition.manifest);
    manifest.selections.push(ProjectPluginSelection {
        id: provider_package_id.to_string(),
        enabled: false,
        required: false,
        target_modes: feature_selection.target_modes,
        packaging: feature_selection.packaging,
        runtime_crate: feature_selection.runtime_crate,
        editor_crate: feature_selection.editor_crate,
        features: Vec::new(),
    });
}
