use std::fmt;

use crate::core::framework::project::ProjectPluginManifest;

use super::tokens::{
    is_lowercase_project_feature_namespace, is_lowercase_project_feature_segment,
    is_lowercase_project_plugin_package_token,
};
use super::ProjectPluginManifestValidationProjection;

pub(in crate::plugin::export_build_plan) fn project_plugin_package_id_diagnostics(
    manifest: &ProjectPluginManifest,
    projection: &ProjectPluginManifestValidationProjection,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    for (selection_index, selection) in manifest.selections.iter().enumerate() {
        if !projection.selection_is_consumed_by_target(selection_index) {
            continue;
        }
        if projection.selection_package_id_is_valid(selection_index) {
            continue;
        }
        let first_diagnostic = diagnostics.len();
        validate_project_plugin_package_id(
            format_args!("project plugin selection id"),
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
    projection: &ProjectPluginManifestValidationProjection,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    for (selection_index, owner) in manifest.selections.iter().enumerate() {
        if !projection.selection_is_consumed_by_target(selection_index) {
            continue;
        }
        for (feature_index, feature) in owner.features.iter().enumerate() {
            if !projection.feature_is_consumed_by_target(selection_index, feature_index) {
                continue;
            }
            if projection.feature_id_is_valid(selection_index, feature_index) {
                continue;
            }
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
    projection: &ProjectPluginManifestValidationProjection,
) {
    let selections = std::mem::take(&mut manifest.selections);
    manifest.selections = selections
        .into_iter()
        .enumerate()
        .filter_map(|(selection_index, mut selection)| {
            if !projection.selection_retained_after_identity_sanitize(selection_index) {
                return None;
            }
            if projection.selection_is_consumed_by_target(selection_index) {
                let features = std::mem::take(&mut selection.features);
                selection.features = features
                    .into_iter()
                    .enumerate()
                    .filter_map(|(feature_index, feature)| {
                        projection
                            .feature_retained_after_identity_sanitize(
                                selection_index,
                                feature_index,
                            )
                            .then_some(feature)
                    })
                    .collect();
            }
            Some(selection)
        })
        .collect();
}

pub(super) fn validate_project_plugin_package_id(
    context: fmt::Arguments<'_>,
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
    let mut segment_count = 0;
    let mut has_empty_segment = false;
    let mut has_invalid_segment = false;
    for segment in feature_id.split('.') {
        segment_count += 1;
        if segment.is_empty() {
            has_empty_segment = true;
        } else if !is_lowercase_project_feature_segment(segment) {
            has_invalid_segment = true;
        }
    }
    if segment_count < 2 {
        diagnostics.push(format!(
            "project plugin feature id `{feature_id}` must use owner.feature dot namespace form"
        ));
    }
    if has_empty_segment {
        diagnostics.push(format!(
            "project plugin feature id `{feature_id}` must not contain empty namespace segments"
        ));
    }
    if has_invalid_segment {
        diagnostics.push(format!(
            "project plugin feature id `{feature_id}` must contain only lowercase ASCII letters, digits, underscores, and dots"
        ));
    }
    if project_plugin_package_id_is_valid(owner_plugin_id) {
        if !project_feature_id_has_owner(owner_plugin_id, feature_id) {
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
        && project_feature_id_has_owner(owner_plugin_id, feature_id)
}

fn project_feature_id_has_owner(owner_plugin_id: &str, feature_id: &str) -> bool {
    feature_id
        .strip_prefix(owner_plugin_id)
        .is_some_and(|suffix| suffix.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::validate_project_plugin_package_id;

    #[test]
    fn deferred_provider_diagnostic_context_preserves_contract() {
        let mut diagnostics = Vec::new();
        validate_project_plugin_package_id(
            format_args!("project plugin selection id"),
            "Bad__",
            &mut diagnostics,
        );
        validate_project_plugin_package_id(
            format_args!("project plugin feature streaming provider_package_id"),
            "Bad__",
            &mut diagnostics,
        );

        assert_eq!(
            diagnostics,
            vec![
                "project plugin selection id `Bad__` must start with a lowercase ASCII letter".to_string(),
                "project plugin selection id `Bad__` must contain only lowercase ASCII letters, digits, and underscores".to_string(),
                "project plugin selection id `Bad__` must not end with an underscore or contain repeated underscores".to_string(),
                "project plugin feature streaming provider_package_id `Bad__` must start with a lowercase ASCII letter".to_string(),
                "project plugin feature streaming provider_package_id `Bad__` must contain only lowercase ASCII letters, digits, and underscores".to_string(),
                "project plugin feature streaming provider_package_id `Bad__` must not end with an underscore or contain repeated underscores".to_string(),
            ]
        );
    }

    #[test]
    fn project_feature_identity_validation_does_not_allocate_scan_helpers() {
        let source = include_str!("identity.rs");
        let segment_collection = ["split('.')", ".collect::<Vec<_>>()"].concat();
        let formatted_prefix = ["format!(\"{owner_", "plugin_id}.\")"].concat();
        assert!(!source.contains(&segment_collection));
        assert!(!source.contains(&formatted_prefix));
    }

    #[test]
    fn project_feature_owner_matching_preserves_the_dot_boundary() {
        assert!(super::project_feature_id_has_owner(
            "rendering",
            "rendering.deferred"
        ));
        assert!(!super::project_feature_id_has_owner(
            "render",
            "rendering.deferred"
        ));
    }
}
