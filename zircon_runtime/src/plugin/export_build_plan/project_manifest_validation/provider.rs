use crate::{plugin::ProjectPluginManifest, RuntimeTargetMode};

use super::identity::{project_plugin_package_id_is_valid, validate_project_plugin_package_id};

pub(in crate::plugin::export_build_plan) fn project_feature_provider_package_id_diagnostics(
    manifest: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    for owner in manifest
        .selections
        .iter()
        .filter(|selection| selection.enabled && selection.supports_target(target))
    {
        for feature in owner
            .features
            .iter()
            .filter(|feature| feature.enabled && feature.supports_target(target))
        {
            let Some(provider_package_id) = feature.provider_package_id.as_deref() else {
                continue;
            };
            let first_diagnostic = diagnostics.len();
            validate_project_plugin_package_id(
                &format!("project plugin feature {} provider_package_id", feature.id),
                provider_package_id,
                &mut diagnostics,
            );
            if feature.required {
                fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}

pub(in crate::plugin::export_build_plan) fn sanitize_invalid_project_provider_package_overrides(
    manifest: &mut ProjectPluginManifest,
    target: RuntimeTargetMode,
) {
    for selection in manifest
        .selections
        .iter_mut()
        .filter(|selection| selection.enabled && selection.supports_target(target))
    {
        for feature in selection
            .features
            .iter_mut()
            .filter(|feature| feature.enabled && feature.supports_target(target))
        {
            if feature
                .provider_package_id
                .as_deref()
                .is_some_and(|provider_id| !project_plugin_package_id_is_valid(provider_id))
            {
                feature.provider_package_id = None;
            }
        }
    }
}
