use crate::plugin::{PluginFeatureBundleManifest, ProjectPluginSelection};

use super::super::super::feature_registration_report::project_selection_from_feature_manifest;

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
