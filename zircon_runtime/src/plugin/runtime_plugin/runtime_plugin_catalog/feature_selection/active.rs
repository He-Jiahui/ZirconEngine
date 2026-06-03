use crate::plugin::{ProjectPluginFeatureSelection, ProjectPluginManifest};

#[derive(Clone, Debug)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct ActiveFeatureSelection<'a> {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) owner_plugin_id: String,
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) feature:
        &'a ProjectPluginFeatureSelection,
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn active_feature_selections(
    manifest: &ProjectPluginManifest,
) -> Vec<ActiveFeatureSelection<'_>> {
    let mut active = Vec::new();
    for owner_selection in &manifest.selections {
        for feature in &owner_selection.features {
            if feature.enabled {
                active.push(ActiveFeatureSelection {
                    owner_plugin_id: owner_selection.id.clone(),
                    feature,
                });
            }
        }
    }
    active
}
