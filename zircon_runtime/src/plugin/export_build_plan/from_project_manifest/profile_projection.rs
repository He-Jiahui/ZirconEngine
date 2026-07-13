use crate::core::framework::project::{
    ExportProfile, ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};

pub(super) struct ExportProfileProjectionDiagnostics {
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}

pub(super) fn export_profile_selection_diagnostics(
    profile: &ExportProfile,
    manifest: &ProjectPluginManifest,
) -> ExportProfileProjectionDiagnostics {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    if !profile.selected_plugins.is_empty() {
        for selection in manifest.selections.iter().filter(|selection| {
            selection.enabled
                && selection.required
                && selection.supports_target(profile.target_mode)
                && !profile
                    .selected_plugins
                    .iter()
                    .any(|plugin_id| plugin_id == &selection.id)
        }) {
            let diagnostic = format!(
                "export profile {} excludes required plugin {} from target {:?}",
                profile.name, selection.id, profile.target_mode
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
    }

    for plugin_id in &profile.selected_plugins {
        if !manifest
            .selections
            .iter()
            .any(|selection| selection.id == *plugin_id)
        {
            let diagnostic = format!(
                "export profile {} selects plugin {} but the project plugin manifest does not contain it",
                profile.name, plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
    }

    for (owner_plugin_id, feature_ids) in &profile.features {
        let Some(owner) = manifest
            .selections
            .iter()
            .find(|selection| selection.id == *owner_plugin_id)
        else {
            let diagnostic = format!(
                "export profile {} selects features for plugin {} but the project plugin manifest does not contain it",
                profile.name, owner_plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
            continue;
        };
        if !profile.selected_plugins.is_empty()
            && !profile
                .selected_plugins
                .iter()
                .any(|plugin_id| plugin_id == owner_plugin_id)
        {
            let diagnostic = format!(
                "export profile {} selects features for plugin {} but the plugin is not selected by the profile",
                profile.name, owner_plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
        for feature in owner.features.iter().filter(|feature| {
            feature.enabled
                && feature.required
                && feature.supports_target(profile.target_mode)
                && !feature_id_list_selects(owner_plugin_id, &feature.id, feature_ids)
        }) {
            let diagnostic = format!(
                "export profile {} excludes required feature {} from plugin {}",
                profile.name, feature.id, owner_plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
        for feature_id in feature_ids {
            let normalized_feature_id = normalize_profile_feature_id(owner_plugin_id, feature_id);
            if !owner
                .features
                .iter()
                .any(|feature| feature.id == normalized_feature_id)
            {
                let diagnostic = format!(
                    "export profile {} selects feature {} for plugin {} but the project plugin manifest does not contain it",
                    profile.name, feature_id, owner_plugin_id
                );
                diagnostics.push(diagnostic.clone());
                fatal_diagnostics.push(diagnostic);
            }
        }
    }

    ExportProfileProjectionDiagnostics {
        diagnostics,
        fatal_diagnostics,
    }
}

pub(super) fn project_plugins_for_export_profile(
    profile: &ExportProfile,
    manifest: &mut ProjectPluginManifest,
) {
    if !profile.selected_plugins.is_empty() {
        for selection in &mut manifest.selections {
            if !profile
                .selected_plugins
                .iter()
                .any(|plugin_id| plugin_id == &selection.id)
            {
                selection.enabled = false;
                selection.required = false;
                for feature in &mut selection.features {
                    feature.enabled = false;
                    feature.required = false;
                }
                continue;
            }
            selection.enabled = true;
        }
    }

    for selection in &mut manifest.selections {
        apply_profile_feature_projection(profile, selection);
    }
}

fn apply_profile_feature_projection(
    profile: &ExportProfile,
    selection: &mut ProjectPluginSelection,
) {
    let Some(selected_features) = profile.features.get(&selection.id) else {
        return;
    };
    for feature in &mut selection.features {
        feature.enabled = profile_selects_feature(&selection.id, feature, selected_features);
    }
}

fn profile_selects_feature(
    owner_plugin_id: &str,
    feature: &ProjectPluginFeatureSelection,
    selected_features: &[String],
) -> bool {
    selected_features
        .iter()
        .any(|feature_id| normalize_profile_feature_id(owner_plugin_id, feature_id) == feature.id)
}

fn feature_id_list_selects(
    owner_plugin_id: &str,
    feature_id: &str,
    selected_features: &[String],
) -> bool {
    selected_features
        .iter()
        .any(|selected| normalize_profile_feature_id(owner_plugin_id, selected) == feature_id)
}

fn normalize_profile_feature_id(owner_plugin_id: &str, feature_id: &str) -> String {
    if feature_id.contains('.') {
        feature_id.to_string()
    } else {
        format!("{owner_plugin_id}.{feature_id}")
    }
}
