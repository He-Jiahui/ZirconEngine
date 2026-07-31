use std::collections::{HashMap, HashSet};

use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
use crate::plugin::PluginFeatureBundleManifest;

use super::super::super::feature_registration_report::project_selection_from_feature_manifest;
use super::super::feature_definitions::FeatureDefinition;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete_owner_feature_selection(
    owner_selection: &mut ProjectPluginSelection,
    feature: &PluginFeatureBundleManifest,
    provider_package_id: Option<&str>,
    feature_indices: &mut HashMap<String, usize>,
) {
    let mut catalog_selection = project_selection_from_feature_manifest(feature);
    if let Some(provider_package_id) = provider_package_id {
        catalog_selection.provider_package_id = Some(provider_package_id.to_string());
    }
    if let Some(index) = feature_indices.get(&catalog_selection.id).copied() {
        let selection = &mut owner_selection.features[index];
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
    feature_indices.insert(catalog_selection.id.clone(), owner_selection.features.len());
    owner_selection.features.push(catalog_selection);
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete_external_provider_selection(
    manifest: &mut ProjectPluginManifest,
    definition: &FeatureDefinition,
    selected_package_ids: &mut HashSet<String>,
) {
    let Some(provider_package_id) = definition.external_provider_for_owner() else {
        return;
    };
    if selected_package_ids.contains(provider_package_id) {
        return;
    }
    let feature_selection = project_selection_from_feature_manifest(&definition.manifest);
    let selection = ProjectPluginSelection {
        id: provider_package_id.to_string(),
        enabled: false,
        required: false,
        target_modes: feature_selection.target_modes,
        packaging: feature_selection.packaging,
        runtime_crate: feature_selection.runtime_crate,
        editor_crate: feature_selection.editor_crate,
        features: Vec::new(),
    };
    selected_package_ids.insert(selection.id.clone());
    manifest.selections.push(selection);
}
