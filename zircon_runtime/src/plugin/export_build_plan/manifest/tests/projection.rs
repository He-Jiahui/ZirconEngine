use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginFeatureSelection, ProjectPluginManifest,
    ProjectPluginSelection,
};

use super::super::{
    project_duplicate_selection_diagnostics, project_feature_id_diagnostics,
    project_feature_provider_package_id_diagnostics, project_plugin_package_id_diagnostics,
    sanitize_project_identity_rows,
};
use super::{begin_projection_build_observation, ProjectPluginManifestValidationProjection};

#[test]
fn project_manifest_validation_projection_scales_linearly_for_packages_features_and_providers() {
    for package_count in [1, 100, 1_000] {
        for features_per_package in [1, 10, 100] {
            let manifest = linear_projection_fixture(package_count, features_per_package);
            begin_projection_build_observation();
            let projection = ProjectPluginManifestValidationProjection::new(
                &manifest,
                RuntimeTargetMode::ClientRuntime,
            );

            let diagnostic_groups = [
                project_plugin_package_id_diagnostics(&manifest, &projection),
                project_feature_id_diagnostics(&manifest, &projection),
                project_duplicate_selection_diagnostics(&manifest, &projection),
                project_feature_provider_package_id_diagnostics(&manifest, &projection),
            ];
            assert!(diagnostic_groups
                .iter()
                .all(|(diagnostics, fatal)| { diagnostics.is_empty() && fatal.is_empty() }));
            let feature_count = package_count * features_per_package;
            assert_eq!(
                projection.external_feature_selections(&manifest).len(),
                feature_count
            );
            let mut sanitized = manifest.clone();
            sanitize_project_identity_rows(&mut sanitized, &projection);
            assert_eq!(sanitized.selections.len(), package_count * 2);
            assert_eq!(
                sanitized
                    .selections
                    .iter()
                    .map(|selection| selection.features.len())
                    .sum::<usize>(),
                feature_count
            );

            let metrics = projection.metrics();
            assert_eq!(metrics.projection_builds, 1);
            assert_eq!(metrics.selection_rows_indexed, package_count * 2);
            assert_eq!(metrics.feature_rows_indexed, feature_count);
            assert_eq!(metrics.selection_rows_refreshed, package_count * 2);
            assert_eq!(metrics.feature_rows_refreshed, feature_count);
            assert_eq!(
                metrics.lookup_probes,
                package_count * 14 + feature_count * 10
            );
        }
    }
}

#[test]
fn projection_preserves_first_rows_and_reuses_provider_membership() {
    let target = RuntimeTargetMode::ClientRuntime;
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection {
                id: "rendering".to_string(),
                enabled: true,
                required: true,
                target_modes: Vec::new(),
                packaging: ExportPackagingStrategy::SourceTemplate,
                runtime_crate: None,
                editor_crate: None,
                features: vec![ProjectPluginFeatureSelection::new("rendering.deferred")
                    .required(true)
                    .with_provider_package_id("postfx")],
            },
            ProjectPluginSelection {
                id: "rendering".to_string(),
                enabled: true,
                required: false,
                target_modes: Vec::new(),
                packaging: ExportPackagingStrategy::SourceTemplate,
                runtime_crate: None,
                editor_crate: None,
                features: vec![ProjectPluginFeatureSelection::new("rendering.deferred")],
            },
            ProjectPluginSelection {
                id: "postfx".to_string(),
                enabled: true,
                required: false,
                target_modes: Vec::new(),
                packaging: ExportPackagingStrategy::SourceTemplate,
                runtime_crate: None,
                editor_crate: None,
                features: Vec::new(),
            },
        ],
    };

    let mut projection = ProjectPluginManifestValidationProjection::new(&manifest, target);

    assert_eq!(projection.first_selection_index("rendering"), Some(0));
    assert_eq!(
        projection.first_feature_index("rendering", "rendering.deferred"),
        Some(0)
    );
    assert!(projection.provider_is_enabled_for_target("postfx"));
    assert_eq!(
        projection.duplicate_selection_first_required(1),
        Some(true),
        "the duplicate keeps the first row's fatal classification"
    );
    assert_eq!(
        projection.duplicate_feature_first_required(0, 0),
        None,
        "the first feature is not a duplicate"
    );

    let (diagnostics, fatal_diagnostics) =
        project_duplicate_selection_diagnostics(&manifest, &projection);
    assert_eq!(
        diagnostics,
        vec!["project plugin selection id `rendering` is declared more than once"]
    );
    assert_eq!(fatal_diagnostics, diagnostics);

    let external_features = projection.external_feature_selections(&manifest);
    assert_eq!(external_features.len(), 1);
    assert_eq!(external_features[0].0.id, "rendering");
    assert_eq!(external_features[0].1.id, "rendering.deferred");

    let mut provider_disabled = manifest.clone();
    provider_disabled.selections[2].enabled = false;
    projection.refresh(&provider_disabled);
    assert!(!projection.provider_is_enabled_for_target("postfx"));
    assert!(projection
        .external_feature_selections(&provider_disabled)
        .is_empty());
    projection.refresh(&manifest);

    let mut sanitized = manifest.clone();
    sanitize_project_identity_rows(&mut sanitized, &projection);
    assert_eq!(
        sanitized
            .selections
            .iter()
            .map(|selection| selection.id.as_str())
            .collect::<Vec<_>>(),
        vec!["rendering", "postfx"],
        "sanitization preserves the first manifest occurrence and order"
    );
}

#[test]
fn projection_preserves_identity_diagnostic_order_and_fatal_classification() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: "audio".to_string(),
            enabled: true,
            required: false,
            target_modes: Vec::new(),
            packaging: ExportPackagingStrategy::SourceTemplate,
            runtime_crate: None,
            editor_crate: None,
            features: vec![ProjectPluginFeatureSelection::new("audio..mix").required(true)],
        }],
    };
    let projection =
        ProjectPluginManifestValidationProjection::new(&manifest, RuntimeTargetMode::ClientRuntime);

    let (diagnostics, fatal_diagnostics) = project_feature_id_diagnostics(&manifest, &projection);
    assert_eq!(
        diagnostics,
        vec!["project plugin feature id `audio..mix` must not contain empty namespace segments"]
    );
    assert_eq!(fatal_diagnostics, diagnostics);
}

fn linear_projection_fixture(
    package_count: usize,
    features_per_package: usize,
) -> ProjectPluginManifest {
    let mut selections = Vec::with_capacity(package_count * 2);
    for index in 0..package_count {
        let owner_id = format!("owner_{index}");
        let provider_id = format!("provider_{index}");
        selections.push(ProjectPluginSelection {
            id: owner_id.clone(),
            enabled: true,
            required: false,
            target_modes: Vec::new(),
            packaging: ExportPackagingStrategy::SourceTemplate,
            runtime_crate: None,
            editor_crate: None,
            features: (0..features_per_package)
                .map(|feature_index| {
                    ProjectPluginFeatureSelection::new(format!(
                        "{owner_id}.feature_{feature_index}"
                    ))
                    .with_provider_package_id(provider_id.clone())
                })
                .collect(),
        });
        selections.push(ProjectPluginSelection {
            id: provider_id,
            enabled: true,
            required: false,
            target_modes: Vec::new(),
            packaging: ExportPackagingStrategy::SourceTemplate,
            runtime_crate: None,
            editor_crate: None,
            features: Vec::new(),
        });
    }
    ProjectPluginManifest { selections }
}
