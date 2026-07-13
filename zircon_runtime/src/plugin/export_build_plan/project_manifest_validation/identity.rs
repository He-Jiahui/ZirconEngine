use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

use super::tokens::{
    is_lowercase_project_feature_namespace, is_lowercase_project_feature_segment,
    is_lowercase_project_plugin_package_token, target_consumes_feature, target_consumes_selection,
};

pub(in crate::plugin::export_build_plan) fn project_plugin_package_id_diagnostics(
    manifest: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    for selection in manifest
        .selections
        .iter()
        .filter(|selection| selection.enabled && selection.supports_target(target))
    {
        let first_diagnostic = diagnostics.len();
        validate_project_plugin_package_id(
            "project plugin selection id",
            &selection.id,
            &mut diagnostics,
        );
        if selection.required {
            fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
        }
    }
    (diagnostics, fatal_diagnostics)
}

pub(in crate::plugin::export_build_plan) fn project_feature_id_diagnostics(
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
            let first_diagnostic = diagnostics.len();
            validate_project_plugin_feature_id(&owner.id, &feature.id, &mut diagnostics);
            if feature.required {
                fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}

pub(in crate::plugin::export_build_plan) fn sanitize_project_identity_rows(
    manifest: &mut ProjectPluginManifest,
    target: RuntimeTargetMode,
) {
    let mut seen_selection_ids = Vec::new();
    manifest.selections.retain(|selection| {
        if !target_consumes_selection(selection, target) {
            return true;
        }
        if !project_plugin_package_id_is_valid(&selection.id) {
            return false;
        }
        if seen_selection_ids.iter().any(|seen| seen == &selection.id) {
            return false;
        }
        seen_selection_ids.push(selection.id.clone());
        true
    });
    for selection in &mut manifest.selections {
        if !target_consumes_selection(selection, target) {
            continue;
        }
        let owner_id = selection.id.clone();
        let mut seen_feature_ids = Vec::new();
        selection.features.retain(|feature| {
            if !target_consumes_feature(feature, target) {
                return true;
            }
            if !project_plugin_feature_id_is_valid(&owner_id, &feature.id) {
                return false;
            }
            if seen_feature_ids.iter().any(|seen| seen == &feature.id) {
                return false;
            }
            seen_feature_ids.push(feature.id.clone());
            true
        });
    }
}

pub(super) fn validate_project_plugin_package_id(
    context: &str,
    package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if package_id.trim().is_empty() || package_id.trim() != package_id {
        diagnostics.push(format!(
            "{context} `{package_id}` must be non-empty and trimmed"
        ));
    }
    if !package_id
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase())
    {
        diagnostics.push(format!(
            "{context} `{package_id}` must start with a lowercase ASCII letter"
        ));
    }
    if !is_lowercase_project_plugin_package_token(package_id) {
        diagnostics.push(format!(
            "{context} `{package_id}` must contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
    if package_id.ends_with('_') || package_id.contains("__") {
        diagnostics.push(format!(
            "{context} `{package_id}` must not end with an underscore or contain repeated underscores"
        ));
    }
}

fn validate_project_plugin_feature_id(
    owner_plugin_id: &str,
    feature_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if feature_id.trim().is_empty() || feature_id.trim() != feature_id {
        diagnostics.push(format!(
            "project plugin feature id `{feature_id}` must be non-empty and trimmed"
        ));
    }
    let segments = feature_id.split('.').collect::<Vec<_>>();
    if segments.len() < 2 {
        diagnostics.push(format!(
            "project plugin feature id `{feature_id}` must use owner.feature dot namespace form"
        ));
    }
    if segments.iter().any(|segment| segment.is_empty()) {
        diagnostics.push(format!(
            "project plugin feature id `{feature_id}` must not contain empty namespace segments"
        ));
    }
    if segments
        .iter()
        .any(|segment| !segment.is_empty() && !is_lowercase_project_feature_segment(segment))
    {
        diagnostics.push(format!(
            "project plugin feature id `{feature_id}` must contain only lowercase ASCII letters, digits, underscores, and dots"
        ));
    }
    if project_plugin_package_id_is_valid(owner_plugin_id) {
        let owner_prefix = format!("{owner_plugin_id}.");
        if !feature_id.starts_with(&owner_prefix) {
            diagnostics.push(format!(
                "project plugin feature id `{feature_id}` must be prefixed by project plugin `{owner_plugin_id}`"
            ));
        }
    }
}

pub(in crate::plugin::export_build_plan) fn project_plugin_package_id_is_valid(
    package_id: &str,
) -> bool {
    !package_id.trim().is_empty()
        && package_id.trim() == package_id
        && package_id
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && is_lowercase_project_plugin_package_token(package_id)
        && !package_id.ends_with('_')
        && !package_id.contains("__")
}

pub(in crate::plugin::export_build_plan) fn project_plugin_feature_id_is_valid(
    owner_plugin_id: &str,
    feature_id: &str,
) -> bool {
    project_plugin_package_id_is_valid(owner_plugin_id)
        && feature_id.trim() == feature_id
        && is_lowercase_project_feature_namespace(feature_id)
        && feature_id.starts_with(&format!("{owner_plugin_id}."))
}
